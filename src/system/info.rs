use super::ascii_logo_with_distro;
#[cfg(target_os = "macos")]
use libc;
use std::env;
#[cfg(target_os = "linux")]
use std::fs;
use std::process::Command;
use std::thread;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

fn detect_host_model() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("hw.model")
            .output()
            && out.status.success()
            && let Ok(s) = String::from_utf8(out.stdout)
        {
            let s = s.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        for path in [
            "/sys/devices/virtual/dmi/id/product_name",
            "/sys/devices/virtual/dmi/id/board_name",
        ] {
            if let Ok(s) = std::fs::read_to_string(path)
                && !s.trim().is_empty()
                && s.trim() != "To Be Filled By O.E.M."
            {
                return Some(s.trim().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn detect_gpu_and_resolution() -> (Option<String>, Option<String>) {
    let Ok(out) = Command::new("/usr/sbin/system_profiler")
        .args(["SPDisplaysDataType", "-detailLevel", "mini"])
        .output()
    else {
        return (None, None);
    };
    if !out.status.success() {
        return (None, None);
    }
    let Ok(text) = String::from_utf8(out.stdout) else {
        return (None, None);
    };
    let mut gpu_info: Option<String> = None;
    let mut resolution: Option<String> = None;
    for line in text.lines() {
        let l = line.trim();
        if gpu_info.is_none() {
            if l.starts_with("Chipset Model:") {
                gpu_info = Some(l.replace("Chipset Model:", "").trim().to_string());
            } else if l.starts_with("Graphics:") {
                gpu_info = Some(l.replace("Graphics:", "").trim().to_string());
            }
        }
        if resolution.is_none() && l.starts_with("Resolution:") {
            resolution = Some(l.replace("Resolution:", "").trim().to_string());
        }
        if gpu_info.is_some() && resolution.is_some() {
            break;
        }
    }
    (gpu_info, resolution)
}

#[cfg(target_os = "linux")]
fn detect_gpu_and_resolution() -> (Option<String>, Option<String>) {
    let gpu_info = Command::new("lspci")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|text| {
            text.lines().find_map(|line| {
                let lower = line.to_ascii_lowercase();
                if lower.contains("vga") || lower.contains("3d controller") {
                    if let Some(pos) = line.find(':') {
                        Some(line[pos + 1..].trim().to_string())
                    } else {
                        Some(line.trim().to_string())
                    }
                } else {
                    None
                }
            })
        });

    let resolution = Command::new("xrandr")
        .arg("--query")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|text| {
            text.lines().find_map(|line| {
                if line.contains(" connected primary") || line.contains(" connected ") {
                    for part in line.split_whitespace() {
                        if part.contains('x')
                            && part
                                .chars()
                                .all(|c| c.is_ascii_digit() || c == 'x' || c == '+')
                            && part.contains('+')
                        {
                            return Some(part.split('+').next().unwrap().to_string());
                        }
                    }
                }
                None
            })
        });

    (gpu_info, resolution)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn detect_gpu_and_resolution() -> (Option<String>, Option<String>) {
    (None, None)
}

fn detect_battery() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("pmset").args(["-g", "batt"]).output()
            && out.status.success()
            && let Ok(text) = String::from_utf8(out.stdout)
        {
            for line in text.lines() {
                let Some((pct_part, _)) = line.split_once('%') else {
                    continue;
                };
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
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with("BAT") || name.to_ascii_lowercase().contains("battery") {
                    let base = e.path();
                    let cap = fs::read_to_string(base.join("capacity")).ok();
                    let stat = fs::read_to_string(base.join("status")).ok();
                    if let Some(cap_trim) = cap.as_deref().map(str::trim).filter(|s| !s.is_empty())
                    {
                        let s = stat.unwrap_or_default();
                        return Some(format!("{}% {}", cap_trim, s.trim()));
                    }
                }
            }
        }
    }
    None
}

fn detect_pkg_count() -> Option<String> {
    use std::process::Command;
    let candidates: &[(&str, &[&str], &str)] = &[
        ("brew", &["list"], "brew"),
        ("pacman", &["-Q"], "pacman"),
        ("dpkg-query", &["-f", "${binary:Package}\n", "-W"], "dpkg"),
        ("apt", &["list", "--installed"], "apt"),
        ("rpm", &["-qa"], "rpm"),
        ("flatpak", &["list"], "flatpak"),
    ];
    for (cmd, args, label) in candidates {
        if let Ok(out) = Command::new(cmd).args(*args).output()
            && out.status.success()
            && let Ok(text) = String::from_utf8(out.stdout)
        {
            let count = text.lines().filter(|line| !line.trim().is_empty()).count();
            if count > 0 {
                return Some(format!("{} ({} pkgs)", label, count));
            }
        }
    }
    None
}

fn detect_temperature() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let mut temps = Vec::new();
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for e in entries.flatten() {
                let p = e.path();
                if p.join("type").exists()
                    && p.join("temp").exists()
                    && let Ok(t) = fs::read_to_string(p.join("temp"))
                    && let Ok(v) = t.trim().parse::<i64>()
                    && v > 0
                {
                    temps.push(v as f64 / 1000.0);
                }
            }
        }
        if temps.is_empty()
            && let Ok(entries) = fs::read_dir("/sys/class/hwmon")
        {
            for e in entries.flatten() {
                let p = e.path();
                for idx in 1..=5 {
                    let file = p.join(format!("temp{}{}_input", idx, ""));
                    if let Ok(t) = fs::read_to_string(&file)
                        && let Ok(v) = t.trim().parse::<i64>()
                        && v > 0
                    {
                        temps.push(v as f64 / 1000.0);
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

fn detect_uptime_secs() -> u64 {
    #[cfg(target_os = "macos")]
    unsafe {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        if libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) == 0 && ts.tv_sec > 0 {
            return ts.tv_sec as u64;
        }
    }

    let uptime = System::uptime();
    if uptime < 366 * 24 * 60 * 60 {
        return uptime;
    }

    let boot_time = System::boot_time();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    if boot_time > 0 && now > boot_time {
        now - boot_time
    } else {
        uptime
    }
}

fn detect_cpu_base_freq_ghz(sys: &System, brand: &str) -> Option<f64> {
    let freqs: Vec<u64> = sys
        .cpus()
        .iter()
        .map(|c| c.frequency())
        .filter(|f| *f > 0)
        .collect();
    if !freqs.is_empty() {
        let avg_mhz = freqs.iter().sum::<u64>() as f64 / freqs.len() as f64;
        if avg_mhz > 100.0 {
            return Some(avg_mhz / 1000.0);
        }
    }

    #[cfg(target_os = "macos")]
    {
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
                && hz > 0
            {
                return Some(hz as f64 / 1_000_000_000.0);
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
                && hz_max > 0
            {
                return Some(hz_max as f64 / 1_000_000_000.0);
            }
        }
        if let Ok(out) = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("hw.cpufrequency")
            .output()
            && out.status.success()
            && let Ok(s) = String::from_utf8(out.stdout)
            && let Ok(hz) = s.trim().parse::<u64>()
            && hz > 0
        {
            return Some(hz as f64 / 1_000_000_000.0);
        }
        if let Ok(out) = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("hw.cpufrequency_max")
            .output()
            && out.status.success()
            && let Ok(s) = String::from_utf8(out.stdout)
            && let Ok(hz) = s.trim().parse::<u64>()
            && hz > 0
        {
            return Some(hz as f64 / 1_000_000_000.0);
        }
        if let Ok(out) = Command::new("sysctl").arg("hw.cpufrequency").output()
            && out.status.success()
            && let Ok(s) = String::from_utf8(out.stdout)
            && let Some(val) = s.split(':').nth(1)
            && let Ok(hz) = val.trim().parse::<u64>()
            && hz > 0
        {
            return Some(hz as f64 / 1_000_000_000.0);
        }
        if let Ok(out) = Command::new("sysctl").arg("hw.cpufrequency_max").output()
            && out.status.success()
            && let Ok(s) = String::from_utf8(out.stdout)
            && let Some(val) = s.split(':').nth(1)
            && let Ok(hz) = val.trim().parse::<u64>()
            && hz > 0
        {
            return Some(hz as f64 / 1_000_000_000.0);
        }
    }

    #[cfg(target_os = "linux")]
    {
        for path in [
            "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
        ] {
            if let Ok(s) = fs::read_to_string(path)
                && let Ok(khz) = s.trim().parse::<u64>()
                && khz > 0
            {
                return Some(khz as f64 / 1_000_000.0);
            }
        }
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.to_ascii_lowercase().starts_with("cpu mhz")
                    && let Some(rest) = line.split(':').nth(1)
                    && let Ok(mhz) = rest.trim().parse::<f64>()
                    && mhz > 100.0
                {
                    return Some(mhz / 1000.0);
                }
            }
        }
    }

    let mut best: Option<f64> = None;
    let lower = brand.to_ascii_lowercase();
    for token in lower.split_whitespace() {
        if let Some(pos) = token.find("ghz") {
            let num = &token[..pos];
            if let Ok(v) = num
                .replace(|c: char| !c.is_ascii_digit() && c != '.', "")
                .parse::<f64>()
                && v > 0.1
            {
                best = Some(best.map(|b| b.max(v)).unwrap_or(v));
            }
        } else if let Some(pos) = token.find("mhz") {
            let num = &token[..pos];
            if let Ok(v) = num
                .replace(|c: char| !c.is_ascii_digit() && c != '.', "")
                .parse::<f64>()
                && v > 100.0
            {
                let ghz = v / 1000.0;
                best = Some(best.map(|b| b.max(ghz)).unwrap_or(ghz));
            }
        }
    }
    best
}

pub const INFO_FIELD_KEYS: &[&str] = &[
    "header",
    "os",
    "host",
    "kernel",
    "uptime",
    "shell",
    "terminal",
    "cpu",
    "cores",
    "gpu",
    "resolution",
    "battery",
    "packages",
    "temperature",
    "memory",
    "swap",
    "disk",
    "network",
    "locale",
];

#[derive(Clone, Debug)]
pub enum InfoFieldSelection {
    All,
    Show(Vec<&'static str>),
    Hide(Vec<&'static str>),
}

impl InfoFieldSelection {
    fn includes(&self, key: &str) -> bool {
        match self {
            Self::All => true,
            Self::Show(keys) => keys.contains(&key),
            Self::Hide(keys) => !keys.contains(&key),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SystemInfoOptions {
    pub show_logo: bool,
    pub fields: InfoFieldSelection,
    pub logo_override: Option<Vec<String>>,
    pub distro_id: Option<String>,
}

impl SystemInfoOptions {
    pub fn new(show_logo: bool, fields: InfoFieldSelection) -> Self {
        Self {
            show_logo,
            fields,
            logo_override: None,
            distro_id: None,
        }
    }

    pub fn with_logo_override(mut self, logo_override: Option<Vec<String>>) -> Self {
        self.logo_override = logo_override;
        self
    }

    pub fn with_distro_id(mut self, distro_id: Option<String>) -> Self {
        self.distro_id = distro_id;
        self
    }
}

#[derive(Clone, Debug)]
pub struct SystemInfoField {
    pub key: &'static str,
    pub line: String,
}

pub fn info_field_key(input: &str) -> Option<&'static str> {
    INFO_FIELD_KEYS.iter().copied().find(|key| *key == input)
}

fn order_fields(
    fields: Vec<SystemInfoField>,
    selection: &InfoFieldSelection,
) -> Vec<SystemInfoField> {
    match selection {
        InfoFieldSelection::Show(keys) => keys
            .iter()
            .filter_map(|key| fields.iter().find(|field| field.key == *key).cloned())
            .collect(),
        InfoFieldSelection::All | InfoFieldSelection::Hide(_) => fields,
    }
}

pub fn generate_system_info_fields(options: &SystemInfoOptions) -> Vec<SystemInfoField> {
    let selection = &options.fields;

    let host_handle = selection
        .includes("host")
        .then(|| thread::spawn(detect_host_model));
    let gpu_handle = (selection.includes("gpu") || selection.includes("resolution"))
        .then(|| thread::spawn(detect_gpu_and_resolution));
    let battery_handle = selection
        .includes("battery")
        .then(|| thread::spawn(detect_battery));
    let pkg_handle = selection
        .includes("packages")
        .then(|| thread::spawn(detect_pkg_count));
    let temp_handle = selection
        .includes("temperature")
        .then(|| thread::spawn(detect_temperature));

    let needs_sys = selection.includes("cpu")
        || selection.includes("cores")
        || selection.includes("memory")
        || selection.includes("swap");
    let sys = needs_sys.then(|| {
        let sys_refreshes = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_frequency())
            .with_memory(MemoryRefreshKind::everything());
        System::new_with_specifics(sys_refreshes)
    });

    let mut fields = Vec::new();

    if selection.includes("header") {
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "hostname".to_string());
        fields.push(SystemInfoField {
            key: "header",
            line: format!("{}@{}\n-------", username, hostname),
        });
    }

    if selection.includes("os") {
        let line = if let Some(os_name) = System::name() {
            if let Some(os_version) = System::os_version() {
                format!(
                    "OS: {} {} ({})",
                    os_name,
                    os_version,
                    std::env::consts::ARCH
                )
            } else {
                format!("OS: {} ({})", os_name, std::env::consts::ARCH)
            }
        } else {
            "OS: Unknown".to_string()
        };
        fields.push(SystemInfoField { key: "os", line });
    }

    if selection.includes("host") {
        let host_line = host_handle
            .and_then(|handle| handle.join().ok())
            .flatten()
            .unwrap_or_else(|| "Unknown".to_string());
        fields.push(SystemInfoField {
            key: "host",
            line: format!("Host: {}", host_line),
        });
    }

    if selection.includes("kernel")
        && let Some(kernel_version) = System::kernel_version()
    {
        fields.push(SystemInfoField {
            key: "kernel",
            line: format!("Kernel: {}", kernel_version),
        });
    }

    if selection.includes("uptime") {
        let uptime = detect_uptime_secs();
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        fields.push(SystemInfoField {
            key: "uptime",
            line: format!("Uptime: {} hours, {} mins", hours, minutes),
        });
    }

    if selection.includes("shell") {
        let shell = env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
        let shell_name = shell.split('/').next_back().unwrap_or("unknown");
        fields.push(SystemInfoField {
            key: "shell",
            line: format!("Shell: {}", shell_name),
        });
    }

    if selection.includes("terminal") {
        let terminal = env::var("TERM_PROGRAM")
            .or_else(|_| env::var("TERMINAL"))
            .unwrap_or_else(|_| "unknown".to_string());
        fields.push(SystemInfoField {
            key: "terminal",
            line: format!("Terminal: {}", terminal),
        });
    }

    if let Some(sys) = &sys {
        if !sys.cpus().is_empty() {
            let cpu_count = sys.cpus().len();
            let brand_primary = sys
                .cpus()
                .iter()
                .find(|c| !c.brand().trim().is_empty())
                .map(|c| c.brand().to_string())
                .filter(|s| s.chars().any(|ch| ch.is_alphanumeric()))
                .unwrap_or_else(|| "Unknown CPU".to_string());
            if selection.includes("cpu") {
                let arch = std::env::consts::ARCH;
                let freq_ghz = detect_cpu_base_freq_ghz(sys, &brand_primary);
                let freq_part = freq_ghz
                    .map(|v| format!(" @ {:.2} GHz", v))
                    .unwrap_or_default();
                fields.push(SystemInfoField {
                    key: "cpu",
                    line: format!(
                        "CPU: {} ({} cores, {}){}",
                        brand_primary.trim(),
                        cpu_count,
                        arch,
                        freq_part
                    ),
                });
            }
            if selection.includes("cores") {
                let line = if let Some(phys) = System::physical_core_count() {
                    if phys != cpu_count {
                        format!("Cores: {} physical / {} logical", phys, cpu_count)
                    } else {
                        format!("Cores: {} logical", cpu_count)
                    }
                } else {
                    format!("Cores: {} logical", cpu_count)
                };
                fields.push(SystemInfoField { key: "cores", line });
            }
        } else if selection.includes("cpu") {
            fields.push(SystemInfoField {
                key: "cpu",
                line: "CPU: Unknown".to_string(),
            });
        }
    }

    if let Some(handle) = gpu_handle {
        let (gpu_info, resolution) = handle.join().ok().unwrap_or((None, None));
        if selection.includes("gpu") {
            fields.push(SystemInfoField {
                key: "gpu",
                line: format!("GPU: {}", gpu_info.unwrap_or_else(|| "Unknown".to_string())),
            });
        }
        if selection.includes("resolution")
            && let Some(res) = resolution
        {
            fields.push(SystemInfoField {
                key: "resolution",
                line: format!("Resolution: {}", res),
            });
        }
    }

    if selection.includes("battery")
        && let Some(batt) = battery_handle
            .and_then(|handle| handle.join().ok())
            .flatten()
    {
        fields.push(SystemInfoField {
            key: "battery",
            line: format!("Battery: {}", batt),
        });
    }

    if selection.includes("packages")
        && let Some(pkgs) = pkg_handle.and_then(|handle| handle.join().ok()).flatten()
    {
        fields.push(SystemInfoField {
            key: "packages",
            line: format!("Packages: {}", pkgs),
        });
    }

    if selection.includes("temperature")
        && let Some(temp) = temp_handle.and_then(|handle| handle.join().ok()).flatten()
    {
        fields.push(SystemInfoField {
            key: "temperature",
            line: format!("Temp: {}", temp),
        });
    }

    if selection.includes("memory")
        && let Some(sys) = &sys
    {
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_percent = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64 * 100.0) as u32
        } else {
            0
        };
        fields.push(SystemInfoField {
            key: "memory",
            line: format!(
                "Memory: {:.2} GiB / {:.2} GiB ({}%)",
                used_memory as f64 / 1024.0 / 1024.0 / 1024.0,
                total_memory as f64 / 1024.0 / 1024.0 / 1024.0,
                memory_percent
            ),
        });
    }

    if selection.includes("swap")
        && let Some(sys) = &sys
    {
        let total_swap = sys.total_swap();
        let line = if total_swap > 0 {
            let used_swap = sys.used_swap();
            format!(
                "Swap: {:.2} GiB / {:.2} GiB",
                used_swap as f64 / 1024.0 / 1024.0 / 1024.0,
                total_swap as f64 / 1024.0 / 1024.0 / 1024.0
            )
        } else {
            "Swap: Disabled".to_string()
        };
        fields.push(SystemInfoField { key: "swap", line });
    }

    if selection.includes("disk") {
        let disks = Disks::new_with_refreshed_list();
        let line = if let Some(disk) = disks
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
            format!(
                "Disk ({}): {:.2} GiB / {:.2} GiB ({}%)",
                disk.mount_point().to_string_lossy(),
                used_gib,
                total_gib,
                pct
            )
        } else {
            "Disk: Unknown".to_string()
        };
        fields.push(SystemInfoField { key: "disk", line });
    }

    if selection.includes("network") {
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
        let line = if let Some((name, ip)) = local_ip {
            format!("Local IP ({}): {}", name, ip)
        } else {
            "Local IP: Unknown".to_string()
        };
        fields.push(SystemInfoField {
            key: "network",
            line,
        });
    }

    if selection.includes("locale") {
        let locale = env::var("LANG").unwrap_or_else(|_| "C".to_string());
        fields.push(SystemInfoField {
            key: "locale",
            line: format!("Locale: {}", locale),
        });
    }

    order_fields(fields, selection)
}

pub fn generate_system_info(options: &SystemInfoOptions) -> Vec<String> {
    let fields = generate_system_info_fields(options);

    let mut logo_lines: Vec<String> = if options.show_logo {
        match options.logo_override.as_deref() {
            Some(lines) => lines.to_vec(),
            None => ascii_logo_with_distro(options.distro_id.as_deref())
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    } else {
        Vec::new()
    };
    if options.logo_override.is_some() {
        pad_custom_logo_lines(&mut logo_lines);
    }
    let show_logo = options.show_logo && !logo_lines.is_empty();

    let mut result = Vec::new();
    let header_first = fields.first().is_some_and(|field| field.key == "header");
    let info_lines = flatten_info_lines(&fields, show_logo && header_first);
    let info_offset = if show_logo && header_first { 2 } else { 0 };
    let max_lines = if show_logo {
        logo_lines.len().max(info_lines.len() + info_offset)
    } else {
        info_lines.len()
    };
    for i in 0..max_lines {
        if show_logo {
            let ascii_part = if i < logo_lines.len() {
                &logo_lines[i]
            } else {
                "                                  "
            };
            let info_part = if i >= info_offset && i - info_offset < info_lines.len() {
                &info_lines[i - info_offset]
            } else {
                ""
            };
            result.push(format!("{}{}", ascii_part, info_part));
        } else {
            let info_part = if i < info_lines.len() {
                &info_lines[i]
            } else {
                ""
            };
            result.push(info_part.to_string());
        }
    }
    result
}

fn flatten_info_lines(
    fields: &[SystemInfoField],
    suppress_logo_header_divider: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    for field in fields {
        if suppress_logo_header_divider && field.key == "header" {
            if let Some(header) = field.line.lines().next() {
                lines.push(header.to_string());
                lines.push(String::new());
            }
        } else {
            lines.extend(field.line.lines().map(str::to_string));
        }
    }
    lines
}

fn pad_custom_logo_lines(lines: &mut [String]) {
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    for line in lines {
        let padding = width.saturating_sub(line.chars().count()) + 2;
        line.push_str(&" ".repeat(padding));
    }
}

pub fn generate_system_info_json(options: &SystemInfoOptions) -> String {
    let fields = generate_system_info_fields(options);
    let mut parts = Vec::with_capacity(fields.len());
    for field in fields {
        let key = serde_json::to_string(field.key).unwrap_or_else(|_| "\"\"".to_string());
        let value = serde_json::to_string(&field.line).unwrap_or_else(|_| "\"\"".to_string());
        parts.push(format!("{}:{}", key, value));
    }
    format!("{{{}}}", parts.join(","))
}
