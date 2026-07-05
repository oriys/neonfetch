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
    std::fs::write(&path, "CYO_LOGO\nNEON_ART\n").expect("failed to write logo file");

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

    let _ = std::fs::remove_file(path);
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
    std::fs::write(&path, "   \n\t\r\n").expect("failed to write logo file");

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

    let _ = std::fs::remove_file(path);
}

#[test]
fn logo_file_normalizes_crlf_tabs_and_ansi_sequences() {
    let path = temp_logo_path("normalize");
    std::fs::write(&path, "\x1b[31mA\tB\x1b[0m\r\nC\tD\n").expect("failed to write logo file");

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

    let _ = std::fs::remove_file(path);
}

fn neonfetch_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_neonfetch"))
}

fn temp_logo_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    path.push(format!(
        "neonfetch-{}-{}-{}.txt",
        name,
        std::process::id(),
        nanos
    ));
    path
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
