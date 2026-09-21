//! `trousseau edit` (3.5.12, step 3.7).
//!
//! Every test points `$EDITOR` at a small fixture script
//! (`tests/editors/fake-editor.sh` on Unix, `fake-editor.cmd` on
//! Windows) that rewrites the scratch file `trousseau edit` hands it
//! according to `TEST_EDIT_ACTION`, instead of a real interactive
//! editor. `TEST_EDIT_PATH_OUT` and `TEST_EDIT_MODE_OUT` are two side
//! channels the script writes to, so a test can observe the scratch
//! file's path and permission bits, which `trousseau edit` itself never
//! prints.
//!
//! The `remove` and `change` fixture actions operate on the fixed keys
//! `a/removeme` and `a/changeme`; a test using them must create a store
//! with exactly those keys.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::{Path, PathBuf};

use common::{Env, json_stdout};

/// A `TROUSSEAU_TEST_NOW` value earlier than [`LATER`] (debug-only, 5.2).
const EARLIER: &str = "2026-01-01T00:00:00Z";
/// A `TROUSSEAU_TEST_NOW` value later than [`EARLIER`].
const LATER: &str = "2026-01-02T00:00:00Z";

/// The fixture editor script's path, for `$EDITOR`.
#[cfg(unix)]
fn editor_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/editors/fake-editor.sh")
}

/// See the Unix [`editor_path`].
#[cfg(windows)]
fn editor_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/editors/fake-editor.cmd")
}

/// `noop` (the editor does not touch the scratch file): `edit` prints
/// `no changes`, exits 0, and the store's ciphertext is byte-for-byte
/// unchanged (no save happened at all).
#[test]
fn noop_prints_no_changes_and_leaves_the_store_byte_identical() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let before = std::fs::read(env.store_path()).expect("read store before edit");

    let assert = env
        .command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "noop")
        .args(["edit"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("no changes"), "stderr was: {stderr:?}");

    let after = std::fs::read(env.store_path()).expect("read store after edit");
    assert_eq!(before, after, "store ciphertext must be unchanged");
}

/// `empty` (the editor truncates the scratch file): `edit` aborts the
/// same way `noop` does, printing `no changes` and leaving the store
/// untouched.
#[test]
fn empty_aborts_and_leaves_the_store_byte_identical() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let before = std::fs::read(env.store_path()).expect("read store before edit");

    let assert = env
        .command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "empty")
        .args(["edit"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("no changes"), "stderr was: {stderr:?}");

    let after = std::fs::read(env.store_path()).expect("read store after edit");
    assert_eq!(before, after, "store ciphertext must be unchanged");
}

/// `add` (the editor appends a new table): the new entry is created with
/// both `created_at` and `updated_at` set to now.
#[test]
fn add_creates_entry_with_both_timestamps_now() {
    let env = Env::new();
    env.init_store();

    env.command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "add")
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["edit"])
        .assert()
        .success();

    let added = json_stdout(
        &env.command_with_identity()
            .args(["get", "added/key", "--json"])
            .output()
            .expect("get added/key"),
    );
    assert_eq!(added["value"], "added-value");
    assert_eq!(added["encoding"], "utf8");
    assert_eq!(added["created_at"], EARLIER);
    assert_eq!(added["updated_at"], EARLIER);
}

/// `remove` (the editor deletes the `["a/removeme"]` table): that entry
/// is gone, and an untouched entry (`z/keep`) keeps its own timestamp.
#[test]
fn remove_deletes_entry_and_leaves_other_entries_timestamps_alone() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "a/removeme"])
        .write_stdin("gone\n")
        .assert()
        .success();
    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "z/keep"])
        .write_stdin("keepvalue\n")
        .assert()
        .success();

    env.command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "remove")
        .env("TROUSSEAU_TEST_NOW", LATER)
        .args(["edit"])
        .assert()
        .success();

    env.command_with_identity()
        .args(["get", "a/removeme"])
        .assert()
        .failure()
        .code(5);

    let keep = json_stdout(
        &env.command_with_identity()
            .args(["get", "z/keep", "--json"])
            .output()
            .expect("get z/keep"),
    );
    assert_eq!(keep["value"], "keepvalue");
    assert_eq!(
        keep["updated_at"], EARLIER,
        "an entry `edit` did not touch must keep its own timestamp"
    );
}

/// `change` (the editor rewrites `["a/changeme"]`'s `value`): that
/// entry's `updated_at` bumps (its `created_at` is kept), and an
/// untouched entry (`z/keep`) keeps its own timestamp.
#[test]
fn change_bumps_updated_at_only_for_the_changed_key() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "a/changeme"])
        .write_stdin("original\n")
        .assert()
        .success();
    env.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "z/keep"])
        .write_stdin("keepvalue\n")
        .assert()
        .success();

    env.command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "change")
        .env("TROUSSEAU_TEST_NOW", LATER)
        .args(["edit"])
        .assert()
        .success();

    let changed = json_stdout(
        &env.command_with_identity()
            .args(["get", "a/changeme", "--json"])
            .output()
            .expect("get a/changeme"),
    );
    assert_eq!(changed["value"], "changed");
    assert_eq!(changed["created_at"], EARLIER, "created_at must be kept");
    assert_eq!(changed["updated_at"], LATER, "updated_at must bump");

    let keep = json_stdout(
        &env.command_with_identity()
            .args(["get", "z/keep", "--json"])
            .output()
            .expect("get z/keep"),
    );
    assert_eq!(keep["value"], "keepvalue");
    assert_eq!(
        keep["updated_at"], EARLIER,
        "an entry `edit` did not touch must keep its own timestamp"
    );
}

/// `corrupt` (the editor leaves invalid TOML) with `--no-input`: `edit`
/// exits 1 without asking to reopen, the store is untouched, and the
/// scratch file is gone.
#[test]
fn corrupt_with_no_input_exits_1_and_leaves_the_store_and_scratch_untouched() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();
    let before = std::fs::read(env.store_path()).expect("read store before edit");

    let path_out = env.path().join("scratch-path.txt");

    env.command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "corrupt")
        .env("TEST_EDIT_PATH_OUT", &path_out)
        .args(["--no-input", "edit"])
        .assert()
        .failure()
        .code(1);

    let after = std::fs::read(env.store_path()).expect("read store after edit");
    assert_eq!(before, after, "store must be untouched on a parse error");

    let scratch_path = std::fs::read_to_string(&path_out).expect("read scratch path side file");
    assert!(
        !Path::new(&scratch_path).exists(),
        "scratch file {scratch_path} must be deleted after edit aborts"
    );
}

/// `fail` (the editor exits 3): `edit` aborts with exit 1 and the store
/// is untouched.
#[test]
fn fail_editor_nonzero_exit_aborts_with_exit_1() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();
    let before = std::fs::read(env.store_path()).expect("read store before edit");

    env.command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "fail")
        .args(["edit"])
        .assert()
        .failure()
        .code(1);

    let after = std::fs::read(env.store_path()).expect("read store after edit");
    assert_eq!(
        before, after,
        "store must be untouched when the editor fails"
    );
}

/// The scratch file's permission bits are `0600` (3.5.12).
#[cfg(unix)]
#[test]
fn scratch_file_mode_is_0600_on_unix() {
    let env = Env::new();
    env.init_store();

    let mode_out = env.path().join("scratch-mode.txt");
    env.command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "noop")
        .env("TEST_EDIT_MODE_OUT", &mode_out)
        .args(["edit"])
        .assert()
        .success();

    let mode = std::fs::read_to_string(&mode_out).expect("read scratch mode side file");
    assert_eq!(mode.trim(), "600");
}

/// The document `export --format toml` produces (a richer fixture: a
/// `utf8` entry with `env`/`description`, and a `base64` entry) reapplies
/// as a no-op through `edit`'s own `noop` path, proving the two commands
/// share one document format end to end (3.5.11, 3.5.12). Mirrors the
/// fixture in `tests/export_import.rs`'s
/// `export_toml_output_parses_with_document_from_toml`, which covers the
/// parsing half of the same format from the `export` side.
#[test]
fn export_toml_fixture_reapplies_as_a_noop_through_edit() {
    let env = Env::new();
    env.init_store();
    env.command_with_identity()
        .args([
            "set",
            "database/password",
            "--env",
            "DATABASE_PASSWORD",
            "--description",
            "Postgres app role",
        ])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let bin_path = env.path().join("server.key");
    std::fs::write(&bin_path, [0x00, 0x01, 0xff]).expect("write binary fixture");
    env.command_with_identity()
        .args(["set", "tls/server.key", "--from-file"])
        .arg(&bin_path)
        .assert()
        .success();

    let before = std::fs::read(env.store_path()).expect("read store before edit");

    let assert = env
        .command_with_identity()
        .env("EDITOR", editor_path())
        .env("TEST_EDIT_ACTION", "noop")
        .args(["edit"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("no changes"), "stderr was: {stderr:?}");

    let after = std::fs::read(env.store_path()).expect("read store after edit");
    assert_eq!(before, after, "store ciphertext must be unchanged");
}
