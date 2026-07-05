use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn runs_main_with_fetch() {
    let output = neonfetch_command()
        .arg("--fetch")
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
}

#[test]
fn logo_file_output_contains_custom_art() {
    let path = temp_logo_path("custom");
    fs::write(&path, "CYO_LOGO\nNEON_ART\n").expect("failed to write logo file");

    let output = neonfetch_command()
        .arg(format!("--logo-file={}", path.display()))
        .arg("--fetch")
        .arg("--no-header")
        .arg("--no-packages")
        .output()
        .expect("failed to run neonfetch binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("CYO_LOGO"));
    assert!(stdout.contains("NEON_ART"));

    let _ = fs::remove_file(path);
}

#[test]
fn missing_logo_file_warns_and_uses_builtin_logo() {
    let path = temp_logo_path("missing");

    let output = neonfetch_command()
        .arg("--logo-file")
        .arg(&path)
        .arg("--fetch")
        .arg("--no-header")
        .arg("--no-packages")
        .output()
        .expect("failed to run neonfetch binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let os_line = stdout
        .lines()
        .find(|line| line.contains("OS:"))
        .expect("expected OS line in stdout");

    assert!(output.status.success());
    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.contains("warning: could not read logo file"));
    assert!(stderr.contains("using built-in logo"));
    assert!(!os_line.starts_with("OS:"));
}

#[test]
fn no_logo_takes_precedence_over_logo_file() {
    let path = temp_logo_path("no-logo-precedence");

    let output = neonfetch_command()
        .arg("--no-logo")
        .arg("--logo-file")
        .arg(&path)
        .arg("--fetch")
        .arg("--no-header")
        .arg("--no-packages")
        .output()
        .expect("failed to run neonfetch binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stdout.lines().next().expect("expected stdout line");

    assert!(output.status.success());
    assert!(stderr.is_empty());
    assert!(first_line.starts_with("OS:"));
}

#[test]
fn blank_logo_file_behaves_like_no_logo() {
    let path = temp_logo_path("blank");
    fs::write(&path, "   \n\t\r\n").expect("failed to write logo file");

    let output = neonfetch_command()
        .arg("--logo-file")
        .arg(&path)
        .arg("--fetch")
        .arg("--no-header")
        .arg("--no-packages")
        .output()
        .expect("failed to run neonfetch binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().expect("expected stdout line");

    assert!(output.status.success());
    assert!(first_line.starts_with("OS:"));

    let _ = fs::remove_file(path);
}

#[test]
fn logo_file_normalizes_crlf_tabs_and_ansi_sequences() {
    let path = temp_logo_path("normalize");
    fs::write(&path, "\x1b[31mA\tB\x1b[0m\r\nC\tD\n").expect("failed to write logo file");

    let output = neonfetch_command()
        .arg("--logo-file")
        .arg(&path)
        .arg("--fetch")
        .arg("--no-header")
        .arg("--no-packages")
        .output()
        .expect("failed to run neonfetch binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("A    B"));
    assert!(stdout.contains("C    D"));
    assert!(!stdout.contains('\r'));
    assert!(!stdout.contains('\t'));
    assert!(!stdout.contains("\x1b[31m"));

    let _ = fs::remove_file(path);
}

#[test]
fn logo_output_suppresses_header_divider_line() {
    let output = neonfetch_command()
        .args(["--fetch", "--no-packages"])
        .output()
        .expect("failed to run neonfetch binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("-------"));

    let lines: Vec<&str> = stdout.lines().collect();
    let header_index = lines
        .iter()
        .position(|line| line.contains('@'))
        .expect("expected header line");
    let os_index = lines
        .iter()
        .position(|line| line.contains("OS:"))
        .expect("expected OS line");
    assert_eq!(os_index, header_index + 1);
}

#[test]
fn no_logo_output_keeps_header_divider_line() {
    let output = neonfetch_command()
        .args(["--fetch", "--no-logo", "--no-packages"])
        .output()
        .expect("failed to run neonfetch binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-------"));
}

#[test]
fn random_style_with_seed_is_reproducible_for_frame_output() {
    let first = neonfetch_command()
        .args(["--frame", "--style", "random", "--seed", "7"])
        .output()
        .expect("failed to run neonfetch binary");
    let second = neonfetch_command()
        .args(["--frame", "--style", "random", "--seed", "7"])
        .output()
        .expect("failed to run neonfetch binary");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(
        normalize_volatile_lines(&first.stdout),
        normalize_volatile_lines(&second.stdout)
    );
}

#[test]
fn forced_arch_distro_fetch_contains_arch_logo() {
    let output = neonfetch_command()
        .args([
            "--distro",
            "arch",
            "--fetch",
            "--no-packages",
            "--no-header",
        ])
        .output()
        .expect("failed to run neonfetch binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("      /_-''    ''-_\\      "),
        "forced arch output should contain the Arch logo"
    );
}

#[test]
fn lists_palettes() {
    let output = neonfetch_command()
        .arg("--list-palettes")
        .output()
        .expect("failed to run neonfetch binary");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("palette list is valid utf8");
    let names: Vec<&str> = stdout.lines().collect();
    assert!(names.len() >= 7, "expected at least 7 palettes: {names:?}");
    for expected in [
        "default",
        "cyberpunk",
        "dracula",
        "pastel",
        "sunset",
        "ocean",
        "mono",
    ] {
        assert!(names.contains(&expected), "missing palette {expected}");
    }
}

#[test]
fn unknown_palette_warns_and_falls_back() {
    let output = neonfetch_command()
        .args(["--fetch", "--palette", "not-a-palette"])
        .output()
        .expect("failed to run neonfetch binary");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr is valid utf8");
    assert!(stderr.contains("unknown palette 'not-a-palette'"));
    assert!(stderr.contains("using default palette"));
}

#[test]
fn hide_omits_selected_labels() {
    let output = neonfetch_command()
        .args(["--fetch", "--no-logo", "--hide", "network,packages"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!stdout.contains("Local IP"));
    assert!(!stdout.contains("Packages:"));
}

#[test]
fn show_orders_selected_lines() {
    let output = neonfetch_command()
        .args(["--fetch", "--no-logo", "--show", "os,cpu,memory"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("OS:"));
    assert!(lines[1].starts_with("CPU:"));
    assert!(lines[2].starts_with("Memory:"));
}

#[test]
fn show_and_hide_conflict_exits_nonzero() {
    let output = neonfetch_command()
        .args(["--fetch", "--show", "os", "--hide", "network"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--show and --hide cannot be used together"));
}

#[test]
fn json_hide_removes_key() {
    let output = neonfetch_command()
        .args(["--json", "--hide", "network"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert!(value.get("network").is_none());
    assert!(value.get("os").is_some());
}

#[test]
fn legacy_no_packages_hides_packages() {
    let output = neonfetch_command()
        .args(["--fetch", "--no-logo", "--no-packages"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!stdout.contains("Packages:"));
}

#[test]
fn list_fields_prints_available_keys() {
    let output = neonfetch_command()
        .arg("--list-fields")
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for key in [
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
    ] {
        assert!(stdout.lines().any(|line| line == key));
    }
}

fn neonfetch_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_neonfetch"))
}

fn temp_logo_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "neonfetch-{name}-{}-{nanos}.txt",
        std::process::id()
    ))
}

fn normalize_volatile_lines(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|line| {
            for label in ["Uptime:", "CPU:", "Memory:", "Swap:", "Battery:", "Temp:"] {
                if let Some(pos) = line.find(label) {
                    return format!("{}{} <volatile>", &line[..pos], label);
                }
            }
            if let Some(pos) = line.find("Disk (") {
                return format!("{}Disk <volatile>", &line[..pos]);
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
