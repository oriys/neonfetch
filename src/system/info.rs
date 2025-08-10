use std::env;
use sysinfo::{Disks, System};
use super::ascii_logo;

pub fn generate_system_info() -> Vec<String> {
    let mut sys = System::new_all();
    // Explicit refresh to ensure CPU list populated on some platforms
    sys.refresh_all();

    let ascii = ascii_logo();
    let info: Vec<String> = ascii.into_iter().map(|s| s.to_string()).collect();

    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());
    let hostname = System::host_name().unwrap_or_else(|| "hostname".to_string());

    let mut info_lines = vec![format!("{}@{}", username, hostname), "-------".to_string()];

    if let Some(os_name) = System::name() {
        if let Some(os_version) = System::os_version() {
            if cfg!(target_os = "macos") {
                info_lines.push(format!("OS: {} {} arm64", os_name, os_version));
            } else {
                info_lines.push(format!("OS: {} {}", os_name, os_version));
            }
        } else {
            info_lines.push(format!("OS: {}", os_name));
        }
    } else {
        info_lines.push("OS: Unknown".to_string());
    }

    if cfg!(target_os = "macos") {
        info_lines.push("Host: Mac Mini (2024)".to_string());
    } else {
        info_lines.push("Host: Unknown".to_string());
    }

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
        // Gather frequency stats (kHz in sysinfo -> MHz). If unavailable (0), omit.
        let freqs: Vec<u64> = sys.cpus().iter().map(|c| c.frequency() as u64).filter(|f| *f > 0).collect();
        let freq_part = if freqs.is_empty() {
            String::new()
        } else {
            let avg = freqs.iter().sum::<u64>() as f64 / freqs.len() as f64; // MHz
            format!(" @ {:.2} GHz", avg / 1000.0)
        };
        info_lines.push(format!(
            "CPU: {} ({} cores, {}){}",
            brand_primary.trim(),
            cpu_count,
            arch,
            freq_part
        ));
    } else {
        info_lines.push("CPU: Unknown".to_string());
    }

    if cfg!(target_os = "macos") {
        info_lines.push("GPU: Apple M4 (10) @ 1.58 GHz [Integrated]".to_string());
    } else {
        info_lines.push("GPU: Unknown".to_string());
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
