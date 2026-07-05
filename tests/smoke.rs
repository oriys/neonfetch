#[test]
fn runs_main_with_fetch() {
    let output = command()
        .arg("--fetch")
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
}

#[test]
fn hide_omits_selected_labels() {
    let output = command()
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
    let output = command()
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
    let output = command()
        .args(["--fetch", "--show", "os", "--hide", "network"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--show and --hide cannot be used together"));
}

#[test]
fn json_hide_removes_key() {
    let output = command()
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
    let output = command()
        .args(["--fetch", "--no-logo", "--no-packages"])
        .output()
        .expect("failed to run neonfetch binary");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!stdout.contains("Packages:"));
}

#[test]
fn list_fields_prints_available_keys() {
    let output = command()
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

fn command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_neonfetch"))
}
