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
fn lists_palettes() {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_neonfetch"))
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
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_neonfetch"))
        .args(["--fetch", "--palette", "not-a-palette"])
        .output()
        .expect("failed to run neonfetch binary");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr is valid utf8");
    assert!(stderr.contains("unknown palette 'not-a-palette'"));
    assert!(stderr.contains("using default palette"));
}
