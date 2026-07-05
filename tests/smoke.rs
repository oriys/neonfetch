#[test]
fn runs_main_with_fetch() {
    // Just ensure binary runs with --fetch quickly without panic.
    // We invoke main logic through spawning a process to avoid infinite loop.
    use std::process::Command;
    let status = Command::new(env!("CARGO_BIN_EXE_neonfetch"))
        .arg("--fetch")
        .status()
        .expect("failed to run neonfetch binary");
    assert!(status.success());
}

#[test]
fn forced_arch_distro_fetch_contains_arch_logo() {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_neonfetch"))
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
