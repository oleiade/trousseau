//! `trousseau ls`, `rm`, and `mv` (3.5.6 through 3.5.8, step 3.3b).
//!
//! Step 3.3 grew past the plan's 900-line budget and was split into
//! `3.3a` (`set`, `get`) and `3.3b` (`ls`, `rm`, `mv`, this file); see
//! this crate's PR description for the split. Every test creates a
//! recipients store with [`common::Env::init_store`], sealed to the
//! fixture SSH identity's recipient only, and unlocks it through
//! [`common::Env::command_with_identity`].

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::ffi::OsString;

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
fn ls_lists_keys_sorted_bytewise() {
    let env = common::Env::new();
    env.init_store();

    for key in ["b", "a", "c"] {
        env.command_with_identity()
            .args(["set", key])
            .write_stdin("v\n")
            .assert()
            .success();
    }

    let output = env
        .command_with_identity()
        .arg("ls")
        .output()
        .expect("run ls");
    assert_eq!(String::from_utf8(output.stdout).expect("utf8"), "a\nb\nc\n");
}

#[test]
fn ls_prefix_is_a_path_prefix_not_a_string_prefix() {
    let env = common::Env::new();
    env.init_store();

    for key in ["data", "data/x", "database/password"] {
        env.command_with_identity()
            .args(["set", key])
            .write_stdin("v\n")
            .assert()
            .success();
    }

    let output = env
        .command_with_identity()
        .args(["ls", "data"])
        .output()
        .expect("run ls data");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["data", "data/x"]);
}

#[test]
fn ls_long_never_contains_a_value() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["set", "secret/key", "--description", "desc"])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .args(["ls", "--long"])
        .output()
        .expect("run ls --long");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("KEY"));
    assert!(stdout.contains("ENC"));
    assert!(stdout.contains("ENV"));
    assert!(stdout.contains("UPDATED"));
    assert!(stdout.contains("DESCRIPTION"));
    assert!(stdout.contains("secret/key"));
    assert!(stdout.contains("desc"));
    assert!(!stdout.contains("s3cr3t"));
}

#[test]
fn ls_json_shape_matches_appendix_5_1() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["set", "k", "--env", "K_ENV", "--description", "d"])
        .write_stdin("v\n")
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .args(["ls", "--json"])
        .output()
        .expect("run ls --json");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    let items = json.as_array().expect("array");
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item["key"], "k");
    assert_eq!(item["encoding"], "utf8");
    assert_eq!(item["env"], "K_ENV");
    assert_eq!(item["description"], "d");
    assert!(item["created_at"].is_string());
    assert!(item["updated_at"].is_string());
    assert!(item.get("value").is_none());
}

#[test]
fn ls_empty_store_is_exit_0_with_empty_output() {
    let env = common::Env::new();
    env.init_store();

    let output = env
        .command_with_identity()
        .arg("ls")
        .output()
        .expect("run ls");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn rm_missing_key_exits_5_and_leaves_store_unchanged() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["set", "a"])
        .write_stdin("1\n")
        .assert()
        .success();

    env.command_with_identity()
        .args(["rm", "a", "missing"])
        .assert()
        .failure()
        .code(5);

    // Nothing was written: `a` is still there.
    env.command_with_identity()
        .args(["get", "a"])
        .assert()
        .success();
}

#[test]
fn rm_force_ignores_missing_keys() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["set", "a"])
        .write_stdin("1\n")
        .assert()
        .success();

    env.command_with_identity()
        .args(["rm", "a", "missing", "--force"])
        .assert()
        .success();

    env.command_with_identity()
        .args(["get", "a"])
        .assert()
        .failure()
        .code(5);
}

#[test]
fn rm_multiple_keys_removes_all_atomically() {
    let env = common::Env::new();
    env.init_store();

    for key in ["a", "b"] {
        env.command_with_identity()
            .args(["set", key])
            .write_stdin("1\n")
            .assert()
            .success();
    }

    let assert = env
        .command_with_identity()
        .args(["rm", "a", "b", "--json"])
        .assert()
        .success();
    let json = json_stdout(assert.get_output());
    assert_eq!(json["removed"], serde_json::json!(["a", "b"]));

    env.command_with_identity()
        .args(["get", "a"])
        .assert()
        .failure()
        .code(5);
    env.command_with_identity()
        .args(["get", "b"])
        .assert()
        .failure()
        .code(5);

    assert_only_store_file(&env);
}

#[test]
fn mv_to_existing_key_exits_8_then_force_overwrites_carrying_metadata() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "old", "--description", "d"])
        .write_stdin("v1\n")
        .assert()
        .success();
    env.command_with_identity()
        .args(["set", "new"])
        .write_stdin("v2\n")
        .assert()
        .success();

    env.command_with_identity()
        .args(["mv", "old", "new"])
        .assert()
        .failure()
        .code(8);

    let assert = env
        .command_with_identity()
        .env("TROUSSEAU_TEST_NOW", LATER)
        .args(["mv", "old", "new", "--force", "--json"])
        .assert()
        .success();
    let json = json_stdout(assert.get_output());
    assert_eq!(json["from"], "old");
    assert_eq!(json["to"], "new");

    env.command_with_identity()
        .args(["get", "old"])
        .assert()
        .failure()
        .code(5);

    let new_json = json_stdout(
        &env.command_with_identity()
            .args(["get", "new", "--json"])
            .output()
            .expect("run get --json"),
    );
    assert_eq!(new_json["value"], "v1");
    assert_eq!(new_json["description"], "d");
    assert_eq!(new_json["created_at"], EARLIER);

    assert_only_store_file(&env);
}
