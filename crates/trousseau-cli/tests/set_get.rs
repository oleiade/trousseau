//! `trousseau set` and `get` (3.5.4, 3.5.5, step 3.3a).
//!
//! Step 3.3 grew past the plan's 900-line budget and was split into
//! `3.3a` (`set`, `get`) and `3.3b` (`ls`, `rm`, `mv`); see this crate's
//! PR description for the split. Every test creates a recipients store
//! with [`common::Env::init_store`], sealed to the fixture SSH
//! identity's recipient only, and unlocks it through
//! [`common::Env::command_with_identity`].

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::ffi::OsString;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::{Duration, Instant};

mod common;

use common::json_stdout;

/// A `TROUSSEAU_TEST_NOW` value earlier than [`LATER`] (debug-only, 5.2).
const EARLIER: &str = "2026-01-01T00:00:00Z";
/// A `TROUSSEAU_TEST_NOW` value later than [`EARLIER`].
const LATER: &str = "2026-01-02T00:00:00Z";

/// Assert that `env`'s project directory contains exactly one entry, the
/// store file `.trousseau`: no leftover atomic-write temp file, and no
/// other stray file (`env`'s `home`/`config`/`data`/`cache`
/// subdirectories, created by [`common::Env::new`], are not store
/// content and are excluded from this check).
fn assert_only_store_file(env: &common::Env) {
    const ENV_SUBDIRS: [&str; 4] = ["home", "config", "data", "cache"];
    let entries: Vec<OsString> = std::fs::read_dir(env.path())
        .expect("read project dir")
        .map(|entry| entry.expect("dir entry").file_name())
        .filter(|name| !ENV_SUBDIRS.iter().any(|dir| name == dir))
        .collect();
    assert_eq!(entries, vec![OsString::from(".trousseau")]);
}

#[test]
fn set_from_piped_stdin_strips_newline_and_get_prints_without_newline() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let piped = env
        .command_with_identity()
        .args(["get", "k"])
        .output()
        .expect("run get");
    assert!(piped.status.success());
    assert_eq!(piped.stdout, b"v");

    let json_output = env
        .command_with_identity()
        .args(["get", "k", "--json"])
        .output()
        .expect("run get --json");
    let json = json_stdout(&json_output);
    assert_eq!(json["encoding"], "utf8");
    assert_eq!(json["value"], "v");
    assert_eq!(json["key"], "k");

    // The write left exactly one store file behind, no leftover temp file.
    assert_only_store_file(&env);
}

#[test]
fn set_with_value_on_command_line_is_refused() {
    let env = common::Env::new();
    env.init_store();

    let output = env
        .command_with_identity()
        .args(["set", "abc", "hunter2"])
        .output()
        .expect("run set abc hunter2");
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("never from the command line"), "{stderr}");
    assert!(!stderr.contains("hunter2"), "{stderr}");

    // Nothing was written: `abc` never made it into the store.
    let ls = env
        .command_with_identity()
        .args(["ls"])
        .output()
        .expect("run ls");
    let stdout = String::from_utf8(ls.stdout).expect("utf8 stdout");
    assert!(!stdout.contains("abc"), "{stdout}");
}

#[test]
fn set_with_value_on_command_line_in_json_mode_reports_a_usage_error() {
    let env = common::Env::new();
    env.init_store();

    let output = env
        .command_with_identity()
        .args(["--json", "set", "abc", "hunter2"])
        .output()
        .expect("run --json set abc hunter2");
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(!stderr.contains("hunter2"), "{stderr}");
    let error: serde_json::Value = serde_json::from_str(&stderr).expect("stderr is JSON");
    assert_eq!(error["error"]["code"], "usage");
}

#[test]
fn set_from_env_missing_variable_exits_1() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["set", "k", "--from-env", "TROUSSEAU_CRUD_TEST_MISSING_VAR"])
        .env_remove("TROUSSEAU_CRUD_TEST_MISSING_VAR")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn set_from_file_binary_stores_base64_and_out_roundtrips_with_force_rule() {
    let env = common::Env::new();
    env.init_store();

    let source_path = env.path().join("source.bin");
    let bytes: [u8; 3] = [0x00, 0x01, 0xff];
    std::fs::write(&source_path, bytes).expect("write source file");

    env.command_with_identity()
        .args(["set", "k", "--from-file"])
        .arg(&source_path)
        .assert()
        .success();

    let json_output = env
        .command_with_identity()
        .args(["get", "k", "--json"])
        .output()
        .expect("run get --json");
    let json = json_stdout(&json_output);
    assert_eq!(json["encoding"], "base64");

    let piped = env
        .command_with_identity()
        .args(["get", "k"])
        .output()
        .expect("run get");
    assert_eq!(piped.stdout, bytes);

    let out_path = env.path().join("out.bin");
    env.command_with_identity()
        .args(["get", "k", "--out"])
        .arg(&out_path)
        .assert()
        .success();
    assert_eq!(std::fs::read(&out_path).expect("read out file"), bytes);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&out_path)
            .expect("stat out file")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    // A second `--out` to the same path refuses (exit 8) without `--force`.
    env.command_with_identity()
        .args(["get", "k", "--out"])
        .arg(&out_path)
        .assert()
        .failure()
        .code(8);

    // `--force` overwrites it.
    env.command_with_identity()
        .args(["get", "k", "--out"])
        .arg(&out_path)
        .arg("--force")
        .assert()
        .success();
    assert_eq!(std::fs::read(&out_path).expect("read out file"), bytes);
}

#[test]
fn set_existing_key_preserves_created_at_and_description_bumps_updated_at() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "k", "--description", "desc"])
        .write_stdin("v1\n")
        .assert()
        .success();

    let first = json_stdout(
        &env.command_with_identity()
            .args(["get", "k", "--json"])
            .output()
            .expect("run get --json"),
    );

    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", LATER)
        .args(["set", "k"])
        .write_stdin("v2\n")
        .assert()
        .success();

    let second = json_stdout(
        &env.command_with_identity()
            .args(["get", "k", "--json"])
            .output()
            .expect("run get --json"),
    );

    assert_eq!(first["created_at"], second["created_at"]);
    assert_ne!(first["updated_at"], second["updated_at"]);
    assert_eq!(second["description"], "desc");
    assert_eq!(second["value"], "v2");
}

#[test]
fn set_no_input_with_piped_stdin_still_reads_stdin_and_succeeds() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["--no-input", "set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .args(["get", "k"])
        .output()
        .expect("run get");
    assert_eq!(output.stdout, b"v");
}

/// Step 3.3's lock test: a `set` blocked reading stdin holds the store's
/// exclusive lock; a concurrent `get` (shared lock) waits and then
/// succeeds once that `set` finishes, while a second `set` with a short
/// lock-wait timeout times out (exit 6) while the lock is still held.
///
/// The plan's test list illustrates the shared-lock reader with `ls`;
/// this uses `get` instead, since `ls` lands in step 3.3b and this test
/// only needs *some* read command to exercise the shared/exclusive lock
/// interaction (see this crate's PR description for the 3.3/3.3a split).
#[test]
fn concurrent_writer_blocks_reader_and_times_out_a_second_writer() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "existing"])
        .write_stdin("v\n")
        .assert()
        .success();

    let bin = assert_cmd::cargo::cargo_bin("trousseau");
    let mut held = std::process::Command::new(&bin)
        .current_dir(env.path())
        .env("HOME", env.home())
        .env("XDG_CONFIG_HOME", env.config_home())
        .env("XDG_DATA_HOME", env.data_home())
        .env("XDG_CACHE_HOME", env.cache_home())
        .env("APPDATA", env.config_home())
        .env("LOCALAPPDATA", env.cache_home())
        .env("USERPROFILE", env.home())
        .env_remove("TROUSSEAU_STORE")
        .env_remove("TROUSSEAU_IDENTITY_FILE")
        .env_remove("TROUSSEAU_PASSPHRASE")
        .env_remove("TROUSSEAU_CONFIG")
        .arg("--identity")
        .arg(common::ssh_identity_path())
        .args(["set", "held"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn the held `set`");

    // Wait until the held `set` has actually *acquired* the lock, not
    // merely created the lock file (`OpenOptions::create` happens before
    // the `flock`/`LockFileEx` attempt, so polling for the file's mere
    // existence races with the held process under scheduler contention:
    // this test flaked under `cargo test`'s parallel execution until it
    // was changed to attempt a real, short-timeout exclusive lock of its
    // own instead, per the charter's "make the test robust to timing").
    let lock_dir = env.lock_dir();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match trousseau::store::lock(
            &env.store_path(),
            &lock_dir,
            trousseau::store::LockMode::Exclusive,
            Duration::from_millis(10),
        ) {
            Err(trousseau::error::Error::LockTimeout) => break,
            Ok(_guard) => {
                // We won the lock ourselves: the held `set` has not
                // acquired it yet. Drop it (releasing immediately) and
                // retry.
            }
            Err(err) => panic!("unexpected error probing the lock: {err}"),
        }
        assert!(
            Instant::now() < deadline,
            "the held `set` never acquired the lock"
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    // A second `set`, with a short lock-wait timeout, must time out while
    // the held `set` still has the lock.
    env.command_with_identity()
        .env("TROUSSEAU_TEST_LOCK_TIMEOUT_MS", "200")
        .args(["set", "other"])
        .write_stdin("v\n")
        .assert()
        .failure()
        .code(6);

    // `get` takes a shared lock and must wait for the held `set` to
    // release its exclusive lock.
    let mut get_cmd = env.command_with_identity();
    get_cmd.args(["get", "existing"]);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let output = get_cmd.output();
        let _ = tx.send(output);
    });

    // `get` should still be waiting a moment later: the held `set`'s
    // stdin is still open.
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        rx.try_recv().is_err(),
        "get should still be waiting for the lock"
    );

    // Release the lock: close the held `set`'s stdin, letting it read
    // EOF and finish.
    drop(held.stdin.take());
    let held_status = held.wait().expect("wait on the held `set`");
    assert!(
        held_status.success(),
        "the held `set` should succeed once its stdin closes"
    );

    let get_output = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("get should finish")
        .expect("get should run");
    assert!(
        get_output.status.success(),
        "get should succeed once the lock is released"
    );
}
