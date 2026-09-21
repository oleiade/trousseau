//! `trousseau run` and `env` (3.5.13, 3.5.14, step 3.6).
//!
//! Every test creates a recipients store with [`common::Env::init_store`],
//! sealed to the fixture SSH identity's recipient only, and unlocks it
//! through [`common::Env::command_with_identity`].
//!
//! Tests that execute a child command build its argv through
//! [`shell_args`], which selects `sh -c` on Unix and `cmd /C` on
//! Windows, so the same test runs on both.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// The `sh`/`cmd` scripts below are plain strings, not `format!` calls;
// their `${VAR}`/`%VAR%` syntax is shell/batch variable expansion, not a
// forgotten formatting argument.
#![allow(clippy::literal_string_with_formatting_args)]

use std::process::Stdio;
use std::time::Duration;

mod common;

/// Build `CMD`'s argv for `run`, portably: `sh -c <unix>` on Unix, `cmd
/// /C <windows>` on Windows.
#[cfg(unix)]
fn shell_args(unix: &str, _windows: &str) -> Vec<String> {
    vec!["sh".to_owned(), "-c".to_owned(), unix.to_owned()]
}

/// Build `CMD`'s argv for `run`, portably: `sh -c <unix>` on Unix, `cmd
/// /C <windows>` on Windows.
#[cfg(windows)]
fn shell_args(_unix: &str, windows: &str) -> Vec<String> {
    vec!["cmd".to_owned(), "/C".to_owned(), windows.to_owned()]
}

#[test]
fn run_prints_injected_variable_via_child_shell() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let mut cmd = env.command_with_identity();
    cmd.arg("run").arg("--");
    cmd.args(shell_args(
        "printf %s \"$DATABASE_PASSWORD\"",
        "echo %DATABASE_PASSWORD%",
    ));
    let output = cmd.output().expect("run `run`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), "s3cr3t");
}

#[test]
fn run_passes_through_the_childs_exit_code() {
    let env = common::Env::new();
    env.init_store();

    let mut cmd = env.command_with_identity();
    cmd.arg("run").arg("--");
    cmd.args(shell_args("exit 7", "exit 7"));
    cmd.assert().failure().code(7);
}

/// `run` has no JSON output (3.5.13): global `--json` is a usage error,
/// exit 2, and `CMD` never runs.
#[test]
fn run_json_is_rejected_with_exit_2() {
    let env = common::Env::new();
    env.init_store();

    let marker = env.path().join("marker");
    let mut cmd = env.command_with_identity();
    cmd.arg("--json").arg("run").arg("--");
    cmd.args(shell_args(
        &format!("touch \"{}\"", marker.display()),
        &format!("type nul > \"{}\"", marker.display()),
    ));
    let assert = cmd.assert().failure().code(2);
    let output = assert.get_output();
    assert!(output.stdout.is_empty(), "stdout should be empty");
    let stderr: serde_json::Value = serde_json::from_slice(&output.stderr).expect("stderr is JSON");
    assert_eq!(stderr["error"]["code"], "usage");
    assert!(!marker.exists(), "run --json must not execute CMD");
}

#[test]
fn run_env_prefix_prefixes_every_injected_name() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let mut cmd = env.command_with_identity();
    cmd.args(["run", "--env-prefix", "APP_", "--"]);
    cmd.args(shell_args(
        "printf %s \"$APP_DATABASE_PASSWORD\"",
        "echo %APP_DATABASE_PASSWORD%",
    ));
    let output = cmd.output().expect("run `run`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), "s3cr3t");
}

/// `[run].env_prefix` in the configuration file (3.4) is the default
/// prefix when `--env-prefix` is absent (fix carried into step 5.1).
#[test]
fn run_honors_env_prefix_from_config_when_flag_is_absent() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let config_dir = env.config_home().join("trousseau");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    std::fs::write(
        config_dir.join("config.toml"),
        "[run]\nenv_prefix = \"APP_\"\n",
    )
    .expect("write config.toml");

    let mut cmd = env.command_with_identity();
    cmd.arg("run").arg("--");
    cmd.args(shell_args(
        "printf %s \"$APP_DATABASE_PASSWORD\"",
        "echo %APP_DATABASE_PASSWORD%",
    ));
    let output = cmd.output().expect("run `run`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), "s3cr3t");
}

#[test]
fn run_only_selects_entries_under_the_path_prefix() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("db-secret\n")
        .assert()
        .success();
    env.command_with_identity()
        .args(["set", "other/key"])
        .write_stdin("other-secret\n")
        .assert()
        .success();

    let mut cmd = env.command_with_identity();
    cmd.args(["run", "--only", "database", "--"]);
    cmd.args(shell_args(
        "printf 'DB=%s OTHER=%s' \"$DATABASE_PASSWORD\" \"${OTHER_KEY:-unset}\"",
        "echo DB=%DATABASE_PASSWORD% OTHER=%OTHER_KEY%",
    ));
    let output = cmd.output().expect("run `run`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DB=db-secret"), "stdout: {stdout}");
    assert!(!stdout.contains("other-secret"), "stdout: {stdout}");
}

#[test]
fn run_no_inherit_drops_a_parent_variable_but_keeps_path() {
    let env = common::Env::new();
    env.init_store();

    let mut cmd = env.command_with_identity();
    cmd.env("TROUSSEAU_RUN_ENV_TEST_VAR", "should-not-appear");
    cmd.args(["run", "--no-inherit", "--"]);
    cmd.args(shell_args(
        "if [ -n \"${TROUSSEAU_RUN_ENV_TEST_VAR:-}\" ]; then echo VAR=leaked; else echo VAR=unset; fi; \
         if [ -n \"${PATH:-}\" ]; then echo PATH=present; else echo PATH=absent; fi",
        "if defined TROUSSEAU_RUN_ENV_TEST_VAR (echo VAR=leaked) else (echo VAR=unset) & \
         if defined PATH (echo PATH=present) else (echo PATH=absent)",
    ));
    let output = cmd.output().expect("run `run`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("VAR=unset"), "custom var leaked: {stdout}");
    assert!(stdout.contains("PATH=present"), "PATH missing: {stdout}");
}

#[test]
fn run_env_conflict_exits_8_and_never_executes_cmd() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "a", "--env", "SAME"])
        .write_stdin("v1\n")
        .assert()
        .success();
    env.command_with_identity()
        .args(["set", "b", "--env", "SAME"])
        .write_stdin("v2\n")
        .assert()
        .success();

    let marker = env.path().join("marker");
    let mut cmd = env.command_with_identity();
    cmd.arg("run").arg("--");
    cmd.args(shell_args(
        &format!("touch \"{}\"", marker.display()),
        &format!("type nul > \"{}\"", marker.display()),
    ));
    cmd.assert().failure().code(8);
    assert!(
        !marker.exists(),
        "an env name conflict must exit before CMD ever runs"
    );
}

#[test]
fn env_skips_a_binary_entry_with_a_warning() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();
    env.command_with_identity()
        .args(["set", "tls/key", "--binary"])
        .write_stdin("binary\n")
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .arg("env")
        .output()
        .expect("run `env`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("skipping binary entry tls/key"),
        "stderr: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DATABASE_PASSWORD"), "stdout: {stdout}");
    assert!(!stdout.contains("TLS_KEY"), "stdout: {stdout}");
}

/// `[run].env_prefix` in the configuration file (3.4) is the default
/// prefix for `env` too, when `--env-prefix` is absent (fix carried
/// into step 5.1).
#[test]
fn env_honors_env_prefix_from_config_when_flag_is_absent() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let config_dir = env.config_home().join("trousseau");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    std::fs::write(
        config_dir.join("config.toml"),
        "[run]\nenv_prefix = \"APP_\"\n",
    )
    .expect("write config.toml");

    let output = env
        .command_with_identity()
        .arg("env")
        .output()
        .expect("run `env`");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("APP_DATABASE_PASSWORD"), "stdout: {stdout}");
}

#[test]
fn env_format_json_and_the_global_json_alias_match() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let via_format = env
        .command_with_identity()
        .args(["env", "--format", "json"])
        .output()
        .expect("run `env --format json`");
    assert!(via_format.status.success());
    let via_format_json: serde_json::Value =
        serde_json::from_slice(&via_format.stdout).expect("stdout is JSON");

    let via_alias = env
        .command_with_identity()
        .args(["env", "--json"])
        .output()
        .expect("run `env --json`");
    assert!(via_alias.status.success());
    let via_alias_json: serde_json::Value =
        serde_json::from_slice(&via_alias.stdout).expect("stdout is JSON");

    assert_eq!(via_format_json, via_alias_json);
    assert_eq!(via_format_json["DATABASE_PASSWORD"], "s3cr3t");
}

/// `sh`'s own `eval` semantics are exercised directly here (not through
/// `trousseau run`, which builds its own child environment independently
/// of `env`'s printed text), so this proves `env`'s shell-format escaping
/// round-trips through a real POSIX shell. Windows' `cmd.exe` has no
/// equivalent `eval`-from-file construct, so this is Unix-only.
#[cfg(unix)]
#[test]
fn env_shell_format_round_trips_a_quote_and_a_newline() {
    let env = common::Env::new();
    env.init_store();
    let value = "a'b\nc";
    env.command_with_identity()
        .args(["set", "k", "--env", "K"])
        .write_stdin(format!("{value}\n"))
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .arg("env")
        .output()
        .expect("run `env`");
    assert!(output.status.success());

    let env_txt = env.path().join("env.txt");
    std::fs::write(&env_txt, &output.stdout).expect("write env.txt");

    let sh_output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "eval \"$(cat '{}')\"; printf %s \"$K\"",
            env_txt.display()
        ))
        .output()
        .expect("run sh");
    assert!(
        sh_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&sh_output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&sh_output.stdout), value);
}

/// Step 3.6's lock test: `run` releases the store's shared lock before
/// its child runs, so a `set` (which needs the exclusive lock) started
/// while the child is still running must not time out.
#[test]
fn run_holds_no_lock_while_the_child_runs() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "database/password"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let mut cmd = env.std_command_with_identity();
    cmd.arg("run").arg("--");
    cmd.args(shell_args("sleep 2", "ping -n 3 127.0.0.1 >nul"));
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let mut child = cmd.spawn().expect("spawn background `run`");

    // Give the background `run` time to unlock the store and start its
    // (roughly two second) child, well before that child exits.
    std::thread::sleep(Duration::from_millis(300));

    // If `run` still held the lock while its child runs, this `set` -
    // with a short lock-wait timeout - would time out (exit 6). It must
    // succeed instead.
    env.command_with_identity()
        .env("TROUSSEAU_TEST_LOCK_TIMEOUT_MS", "200")
        .args(["set", "other"])
        .write_stdin("v\n")
        .assert()
        .success();

    let status = child.wait().expect("wait for the background `run`");
    assert!(status.success(), "the background `run` should exit 0");
}
