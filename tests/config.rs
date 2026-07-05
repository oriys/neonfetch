use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_config_path(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "neonfetch-{name}-{}-{stamp}.toml",
        std::process::id()
    ))
}

fn write_temp_config(name: &str, contents: &str) -> PathBuf {
    let path = unique_config_path(name);
    fs::write(&path, contents).expect("failed to write temp config");
    path
}

fn run_with_env_config(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_neonfetch"))
        .args(args)
        .env("NEONFETCH_CONFIG", path)
        .output()
        .expect("failed to run neonfetch binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout(output),
        stderr(output)
    );
}

#[test]
fn print_config_uses_env_config_file() {
    let path = write_temp_config(
        "env",
        r#"
style = "matrix"
speed = 2.0
duration = 3.5
color_fps = 60
no_logo = true
no_packages = true
no_header = true
mono = true
no_color = true
seed = 42
"#,
    );

    let output = run_with_env_config(&path, &["--print-config"]);
    assert_success(&output);

    let stdout = stdout(&output);
    assert!(stdout.contains("style = \"matrix\""));
    assert!(stdout.contains("speed = 2.0"));
    assert!(stdout.contains("duration = 3.5"));
    assert!(stdout.contains("color_fps = 60"));
    assert!(stdout.contains("no_logo = true"));
    assert!(stdout.contains("no_packages = true"));
    assert!(stdout.contains("no_header = true"));
    assert!(stdout.contains("mono = true"));
    assert!(stdout.contains("no_color = true"));
    assert!(stdout.contains("seed = 42"));

    let _ = fs::remove_file(path);
}

#[test]
fn cli_arguments_override_config_file() {
    let path = write_temp_config(
        "override",
        r#"
style = "matrix"
speed = 2.0
duration = 3.0
color_fps = 60
"#,
    );

    let output = run_with_env_config(
        &path,
        &[
            "--print-config",
            "--style",
            "fire",
            "--speed=3.0",
            "--duration",
            "5.0",
            "--color-fps=45",
        ],
    );
    assert_success(&output);

    let stdout = stdout(&output);
    assert!(stdout.contains("style = \"fire\""));
    assert!(stdout.contains("speed = 3.0"));
    assert!(stdout.contains("duration = 5.0"));
    assert!(stdout.contains("color_fps = 45"));
    assert!(!stdout.contains("style = \"matrix\""));

    let _ = fs::remove_file(path);
}

#[test]
fn no_config_ignores_config_file() {
    let path = write_temp_config(
        "ignored",
        r#"
style = "matrix"
speed = 2.0
no_logo = true
mono = true
seed = 42
"#,
    );

    let output = run_with_env_config(&path, &["--no-config", "--print-config"]);
    assert_success(&output);

    let stdout = stdout(&output);
    assert!(stdout.contains("style = \"neon\""));
    assert!(stdout.contains("speed = 1.0"));
    assert!(stdout.contains("color_fps = 30"));
    assert!(stdout.contains("no_logo = false"));
    assert!(stdout.contains("mono = false"));
    assert!(!stdout.contains("seed = 42"));

    let _ = fs::remove_file(path);
}

#[test]
fn invalid_config_warns_and_uses_defaults() {
    let path = write_temp_config(
        "invalid",
        r#"
style = "matrix"
this is not valid
speed = 2.0
"#,
    );

    let output = run_with_env_config(&path, &["--print-config"]);
    assert_success(&output);

    let stdout = stdout(&output);
    let stderr = stderr(&output);
    assert!(stdout.contains("style = \"neon\""));
    assert!(stdout.contains("speed = 1.0"));
    assert!(stderr.contains("warning"));
    assert!(stderr.contains("ignoring config"));

    let _ = fs::remove_file(path);
}
