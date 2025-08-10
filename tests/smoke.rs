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
