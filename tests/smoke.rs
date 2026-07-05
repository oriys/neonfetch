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
fn random_style_with_seed_is_reproducible_for_frame_output() {
    use std::process::Command;

    let first = Command::new(env!("CARGO_BIN_EXE_neonfetch"))
        .args(["--frame", "--style", "random", "--seed", "7"])
        .output()
        .expect("failed to run neonfetch binary");
    let second = Command::new(env!("CARGO_BIN_EXE_neonfetch"))
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
