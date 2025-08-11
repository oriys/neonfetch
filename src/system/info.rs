use super::ascii_logo;
#[cfg(target_os = "macos")]
use libc;
use std::env;
#[cfg(target_os = "linux")]
use std::fs;
use std::process::Command;
use sysinfo::{Disks, System};

pub fn generate_system_info() -> Vec<String> {
    let mut sys = System::new_all();
    // Explicit refresh to ensure CPU list populated on some platforms
    sys.refresh_all();

    // Helper: host model
    fn detect_host_model() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            // hw.model (e.g. Mac14,12) then try system_profiler for friendly name
            use std::process::Command;
            if let Ok(out) = Command::new("/usr/sbin/sysctl")
                .arg("-n")
                .arg("hw.model")
                .output()
            {
                if out.status.success() {
                    if let Ok(mut s) = String::from_utf8(out.stdout) {
                        s = s.trim().to_string();
                        if !s.is_empty() {
                            return Some(s);
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            // DMI product name
            for path in [
                "/sys/devices/virtual/dmi/id/product_name",
                "/sys/devices/virtual/dmi/id/board_name",
            ] {
                if let Ok(s) = std::fs::read_to_string(path) {
                    let v = s.trim();
                    if !v.is_empty() && v != "To Be Filled By O.E.M." {
                        return Some(v.to_string());
                    }
                }
            }
        }
        None
    }

    // Helper: GPU info (first adapter)
    fn detect_gpu_info() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(out) = Command::new("/usr/sbin/system_profiler")
                .args(["SPDisplaysDataType", "-detailLevel", "mini"])
                .output()
            {
                if out.status.success() {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        for line in text.lines() {
                            let line_trim = line.trim();
                            if line_trim.starts_with("Chipset Model:") {
                                return Some(
                                    line_trim.replace("Chipset Model:", "").trim().to_string(),
                                );
                            }
                            if line_trim.starts_with("Graphics:") {
                                return Some(line_trim.replace("Graphics:", "").trim().to_string());
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            if let Ok(out) = Command::new("lspci").output() {
                if out.status.success() {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        for line in text.lines() {
                            if line.to_ascii_lowercase().contains("vga")
                                || line.to_ascii_lowercase().contains("3d controller")
                            {
                                if let Some(pos) = line.find(':') {
                                    return Some(line[pos + 1..].trim().to_string());
                                } else {
                                    return Some(line.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    let ascii = ascii_logo();
    let info: Vec<String> = ascii.into_iter().map(|s| s.to_string()).collect();

    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());
    let hostname = System::host_name().unwrap_or_else(|| "hostname".to_string());

    let mut info_lines = vec![format!("{}@{}", username, hostname), "-------".to_string()];

    if let Some(os_name) = System::name() {
        if let Some(os_version) = System::os_version() {
            info_lines.push(format!(
                "OS: {} {} ({})",
                os_name,
                os_version,
                std::env::consts::ARCH
            ));
        } else {
            info_lines.push(format!("OS: {} ({})", os_name, std::env::consts::ARCH));
        }
    } else {
        info_lines.push("OS: Unknown".to_string());
    }

    let host_line = detect_host_model().unwrap_or_else(|| "Unknown".to_string());
    info_lines.push(format!("Host: {}", host_line));

    if let Some(kernel_version) = System::kernel_version() {
        info_lines.push(format!("Kernel: {}", kernel_version));
    }

    let uptime = System::uptime();
    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;
    info_lines.push(format!("Uptime: {} hours, {} mins", hours, minutes));

    let shell = env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    let shell_name = shell.split('/').last().unwrap_or("unknown");
    info_lines.push(format!("Shell: {}", shell_name));

    let terminal = env::var("TERM_PROGRAM")
        .or_else(|_| env::var("TERMINAL"))
        .unwrap_or_else(|_| "unknown".to_string());
    info_lines.push(format!("Terminal: {}", terminal));

    if !sys.cpus().is_empty() {
        let cpu_count = sys.cpus().len();
        let brand_primary = sys
            .cpus()
            .iter()
            .find(|c| !c.brand().trim().is_empty())
            .map(|c| c.brand().to_string())
            .filter(|s| s.chars().any(|ch| ch.is_alphanumeric()))
            .unwrap_or_else(|| "Unknown CPU".to_string());
        let arch = std::env::consts::ARCH;
        // Enhanced frequency detection
        fn detect_cpu_base_freq_ghz(sys: &sysinfo::System, brand: &str) -> Option<f64> {
            // 1. Try sysinfo (average of non-zero current frequencies) treat as MHz
            let freqs: Vec<u64> = sys
                .cpus()
                .iter()
                .map(|c| c.frequency() as u64)
                .filter(|f| *f > 0)
                .collect();
            if !freqs.is_empty() {
                let avg_mhz = freqs.iter().sum::<u64>() as f64 / freqs.len() as f64;
                if avg_mhz > 100.0 {
                    // guard against bogus tiny values
                    return Some(avg_mhz / 1000.0);
                }
            }
            // 2. Platform-specific methods
            #[cfg(target_os = "macos")]
            {
                // Prefer direct sysctlbyname (avoids spawning process)
                unsafe {
                    let mut hz: u64 = 0;
                    let mut size = std::mem::size_of::<u64>();
                    let name1 = std::ffi::CString::new("hw.cpufrequency").unwrap();
                    if libc::sysctlbyname(
                        name1.as_ptr(),
                        &mut hz as *mut _ as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) == 0
                    {
                        if hz > 0 {
                            return Some(hz as f64 / 1_000_000_000.0);
                        }
                    }
                    let mut hz_max: u64 = 0;
                    size = std::mem::size_of::<u64>();
                    let name2 = std::ffi::CString::new("hw.cpufrequency_max").unwrap();
                    if libc::sysctlbyname(
                        name2.as_ptr(),
                        &mut hz_max as *mut _ as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) == 0
                    {
                        if hz_max > 0 {
                            return Some(hz_max as f64 / 1_000_000_000.0);
                        }
                    }
                }
                // hw.cpufrequency gives Hz
                if let Ok(out) = Command::new("/usr/sbin/sysctl")
                    .arg("-n")
                    .arg("hw.cpufrequency")
                    .output()
                {
                    if out.status.success() {
                        if let Ok(s) = String::from_utf8(out.stdout) {
                            if let Ok(hz) = s.trim().parse::<u64>() {
                                if hz > 0 {
                                    return Some(hz as f64 / 1_000_000_000.0);
                                }
                            }
                        }
                    }
                }
                if let Ok(out) = Command::new("/usr/sbin/sysctl")
                    .arg("-n")
                    .arg("hw.cpufrequency_max")
                    .output()
                {
                    if out.status.success() {
                        if let Ok(s) = String::from_utf8(out.stdout) {
                            if let Ok(hz) = s.trim().parse::<u64>() {
                                if hz > 0 {
                                    return Some(hz as f64 / 1_000_000_000.0);
                                }
                            }
                        }
                    }
                }
                // Fallback: path-less sysctl (in case /usr/sbin not resolved) and parsing lines
                if let Ok(out) = Command::new("sysctl").arg("hw.cpufrequency").output() {
                    if out.status.success() {
                        if let Ok(s) = String::from_utf8(out.stdout) {
                            if let Some(val) = s.split(':').nth(1) {
                                if let Ok(hz) = val.trim().parse::<u64>() {
                                    if hz > 0 {
                                        return Some(hz as f64 / 1_000_000_000.0);
                                    }
                                }
                            }
                        }
                    }
                }
                if let Ok(out) = Command::new("sysctl").arg("hw.cpufrequency_max").output() {
                    if out.status.success() {
                        if let Ok(s) = String::from_utf8(out.stdout) {
                            if let Some(val) = s.split(':').nth(1) {
                                if let Ok(hz) = val.trim().parse::<u64>() {
                                    if hz > 0 {
                                        return Some(hz as f64 / 1_000_000_000.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            #[cfg(target_os = "linux")]
            {
                // Try sysfs max freq (kHz)
                for path in [
                    "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
                ] {
                    if let Ok(s) = fs::read_to_string(path) {
                        if let Ok(khz) = s.trim().parse::<u64>() {
                            if khz > 0 {
                                return Some(khz as f64 / 1_000_000.0);
                            }
                        }
                    }
                }
                // Fallback /proc/cpuinfo first 'cpu MHz'
                if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
                    for line in content.lines() {
                        if let Some(rest) = line.split(':').nth(1) {
                            if line.to_ascii_lowercase().starts_with("cpu mhz") {
                                if let Ok(mhz) = rest.trim().parse::<f64>() {
                                    if mhz > 100.0 {
                                        return Some(mhz / 1000.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // 3. Parse brand string (look for e.g. '3.20GHz' or '2400 MHz')
            let mut best: Option<f64> = None;
            let lower = brand.to_ascii_lowercase();
            // Simple regex-less scans
            for token in lower.split_whitespace() {
                if let Some(pos) = token.find("ghz") {
                    // pattern like 3.20ghz
                    let num = &token[..pos];
                    if let Ok(v) = num
                        .replace(|c: char| !c.is_ascii_digit() && c != '.', "")
                        .parse::<f64>()
                    {
                        if v > 0.1 {
                            best = Some(best.map(|b| b.max(v)).unwrap_or(v));
                        }
                    }
                } else if let Some(pos) = token.find("mhz") {
                    // 3200mhz
                    let num = &token[..pos];
                    if let Ok(v) = num
                        .replace(|c: char| !c.is_ascii_digit() && c != '.', "")
                        .parse::<f64>()
                    {
                        if v > 100.0 {
                            let ghz = v / 1000.0;
                            best = Some(best.map(|b| b.max(ghz)).unwrap_or(ghz));
                        }
                    }
                }
            }
            best
        }
        let freq_ghz = detect_cpu_base_freq_ghz(&sys, &brand_primary);
        let freq_part = freq_ghz
            .map(|v| format!(" @ {:.2} GHz", v))
            .unwrap_or_default();
        info_lines.push(format!(
            "CPU: {} ({} cores, {}){}",
            brand_primary.trim(),
            cpu_count,
            arch,
            freq_part
        ));
        // Add physical vs logical core detail if available
        if let Some(phys) = sys.physical_core_count() {
            if phys as usize != cpu_count {
                info_lines.push(format!("Cores: {} physical / {} logical", phys, cpu_count));
            }
        } else {
            info_lines.push(format!("Cores: {} logical", cpu_count));
        }
    } else {
        info_lines.push("CPU: Unknown".to_string());
    }

    let gpu_info = detect_gpu_info().unwrap_or_else(|| "Unknown".to_string());
    info_lines.push(format!("GPU: {}", gpu_info));

    // Resolution detection
    fn detect_resolution() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(out) = Command::new("/usr/sbin/system_profiler")
                .args(["SPDisplaysDataType", "-detailLevel", "mini"])
                .output()
            {
                if out.status.success() {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        for line in text.lines() {
                            let l = line.trim();
                            if l.starts_with("Resolution:") {
                                return Some(l.replace("Resolution:", "").trim().to_string());
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            if let Ok(out) = Command::new("xrandr").arg("--query").output() {
                if out.status.success() {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        for line in text.lines() {
                            if line.contains(" connected primary") || line.contains(" connected ") {
                                // pattern: eDP-1 connected primary 2560x1600+0+0 ...
                                for part in line.split_whitespace() {
                                    if part.contains('x')
                                        && part
                                            .chars()
                                            .all(|c| c.is_ascii_digit() || c == 'x' || c == '+')
                                        && part.contains('+')
                                    {
                                        let res = part.split('+').next().unwrap();
                                        return Some(res.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    if let Some(res) = detect_resolution() {
        info_lines.push(format!("Resolution: {}", res));
    }

    // Battery detection
    fn detect_battery() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(out) = Command::new("pmset").args(["-g", "batt"]).output() {
                if out.status.success() {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        for line in text.lines() {
                            if line.contains('%') {
                                if let Some(pct_part) = line.split('%').next() {
                                    // Extract trailing digits
                                    let mut digits = String::new();
                                    for ch in pct_part.chars().rev() {
                                        if ch.is_ascii_digit() {
                                            digits.insert(0, ch);
                                        } else if !digits.is_empty() {
                                            break;
                                        }
                                    }
                                    if !digits.is_empty() {
                                        let status = if line.contains("discharging") {
                                            "discharging"
                                        } else if line.contains("charging") {
                                            "charging"
                                        } else if line.contains("charged") {
                                            "charged"
                                        } else {
                                            ""
                                        };
                                        return Some(format!("{}% {}", digits, status));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            // Look for BAT* directories
            if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if name.starts_with("BAT") || name.to_ascii_lowercase().contains("battery") {
                        let base = e.path();
                        let cap = fs::read_to_string(base.join("capacity")).ok();
                        let stat = fs::read_to_string(base.join("status")).ok();
                        if let Some(cap) = cap {
                            let cap_trim = cap.trim();
                            if !cap_trim.is_empty() {
                                let s = stat.unwrap_or_default();
                                return Some(format!("{}% {}", cap_trim, s.trim()));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    if let Some(batt) = detect_battery() {
        info_lines.push(format!("Battery: {}", batt));
    }

    // Package manager installed count
    fn detect_pkg_count() -> Option<String> {
        use std::process::Command;
        // (cmd, args, name, parse_fn)
        let candidates: &[(&str, &[&str], &str)] = &[
            ("brew", &["list"], "brew"),
            ("pacman", &["-Q"], "pacman"),
            ("dpkg-query", &["-f", "${binary:Package}\n", "-W"], "dpkg"),
            ("apt", &["list", "--installed"], "apt"),
            ("rpm", &["-qa"], "rpm"),
            ("flatpak", &["list"], "flatpak"),
        ];
        for (cmd, args, label) in candidates {
            if let Ok(out) = Command::new(cmd).args(*args).output() {
                if out.status.success() {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        let mut count = 0usize;
                        for line in text.lines() {
                            if !line.trim().is_empty() {
                                count += 1;
                            }
                        }
                        if count > 0 {
                            return Some(format!("{} ({} pkgs)", label, count));
                        }
                    }
                }
            }
        }
        None
    }
    if let Some(pkgs) = detect_pkg_count() {
        info_lines.push(format!("Packages: {}", pkgs));
    }

    // Temperature sensors (simple average / first) Linux only; macOS left N/A for now
    fn detect_temperature() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let mut temps = Vec::new();
            if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.join("type").exists() && p.join("temp").exists() {
                        if let Ok(t) = fs::read_to_string(p.join("temp")) {
                            if let Ok(v) = t.trim().parse::<i64>() {
                                // milliC
                                if v > 0 {
                                    temps.push(v as f64 / 1000.0);
                                }
                            }
                        }
                    }
                }
            }
            if temps.is_empty() {
                // Try hwmon
                if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
                    for e in entries.flatten() {
                        let p = e.path();
                        for idx in 1..=5 {
                            let file = p.join(format!("temp{}{}_input", idx, "")); // build tempX_input
                            if let Ok(t) = fs::read_to_string(&file) {
                                if let Ok(v) = t.trim().parse::<i64>() {
                                    if v > 0 {
                                        temps.push(v as f64 / 1000.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !temps.is_empty() {
                let avg = temps.iter().sum::<f64>() / temps.len() as f64;
                return Some(format!("{:.1}°C", avg));
            }
        }
        None
    }
    if let Some(temp) = detect_temperature() {
        info_lines.push(format!("Temp: {}", temp));
    }

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percent = (used_memory as f64 / total_memory as f64 * 100.0) as u32;
    info_lines.push(format!(
        "Memory: {:.2} GiB / {:.2} GiB ({}%)",
        used_memory as f64 / 1024.0 / 1024.0 / 1024.0,
        total_memory as f64 / 1024.0 / 1024.0 / 1024.0,
        memory_percent
    ));

    let total_swap = sys.total_swap();
    if total_swap > 0 {
        let used_swap = sys.used_swap();
        info_lines.push(format!(
            "Swap: {:.2} GiB / {:.2} GiB",
            used_swap as f64 / 1024.0 / 1024.0 / 1024.0,
            total_swap as f64 / 1024.0 / 1024.0 / 1024.0
        ));
    } else {
        info_lines.push("Swap: Disabled".to_string());
    }

    let disks = Disks::new_with_refreshed_list();
    if let Some(disk) = disks
        .iter()
        .find(|d| {
            let mount = d.mount_point().to_string_lossy();
            mount == "/" || mount == "/System/Volumes/Data"
        })
        .or_else(|| disks.iter().next())
    {
        let total = disk.total_space() as f64;
        let avail = disk.available_space() as f64;
        let used = total - avail;
        let used_gib = used / 1024.0_f64.powi(3);
        let total_gib = total / 1024.0_f64.powi(3);
        let pct = if total > 0.0 {
            (used / total * 100.0).round() as u32
        } else {
            0
        };
        info_lines.push(format!(
            "Disk ({}): {:.2} GiB / {:.2} GiB ({}%)",
            disk.mount_point().to_string_lossy(),
            used_gib,
            total_gib,
            pct
        ));
    } else {
        info_lines.push("Disk: Unknown".to_string());
    }

    let local_ip = get_if_addrs::get_if_addrs().ok().and_then(|ifaces| {
        ifaces.into_iter().find_map(|ifa| {
            if ifa.is_loopback() {
                return None;
            }
            match ifa.ip() {
                std::net::IpAddr::V4(v4) => Some((ifa.name, v4)),
                _ => None,
            }
        })
    });
    if let Some((name, ip)) = local_ip {
        info_lines.push(format!("Local IP ({}): {}", name, ip));
    } else {
        info_lines.push("Local IP: Unknown".to_string());
    }

    let locale = env::var("LANG").unwrap_or_else(|_| "C".to_string());
    info_lines.push(format!("Locale: {}", locale));

    let mut result = Vec::new();
    let max_lines = info.len().max(info_lines.len() + 2);
    for i in 0..max_lines {
        let ascii_part = if i < info.len() {
            &info[i]
        } else {
            "                                  "
        };
        let info_part = if i >= 2 && i - 2 < info_lines.len() {
            &info_lines[i - 2]
        } else {
            ""
        };
        result.push(format!("{}{}", ascii_part, info_part));
    }
    result
}
