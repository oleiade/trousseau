//! `trousseau migrate` (3.5.15, step 3.8).
//!
//! Uses the legacy v0.4 fixtures under
//! `crates/trousseau/tests/fixtures/legacy/` (shared with
//! `crates/trousseau`'s own `legacy.rs` tests): `symmetric-v0.4.json`
//! (AES-256-CFB, passphrase `correct horse battery staple`) and
//! `asymmetric-v0.4.json` (`OpenPGP`, gated on `gpg` being on `PATH`, like
//! step 2.5). Both decrypt to `expected.json`'s four entries; the plain
//! key `easy as` is the only one sanitization renames, to `easy_as`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

use common::{SSH_PUB, json_stdout};

/// The AES-256-CFB legacy fixture's path.
fn symmetric_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../trousseau/tests/fixtures/legacy/symmetric-v0.4.json")
}

/// The `OpenPGP` legacy fixture's path.
#[cfg(unix)]
fn asymmetric_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../trousseau/tests/fixtures/legacy/asymmetric-v0.4.json")
}

/// The throwaway `OpenPGP` secret key used to decrypt [`asymmetric_fixture`].
#[cfg(unix)]
fn test_key_sec() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../trousseau/tests/fixtures/legacy/test-key.sec.asc")
}

/// The legacy passphrase both fixtures' inner document is unrelated to
/// (only the symmetric one is keyed by it): `correct horse battery
/// staple`.
const SYMMETRIC_PASSPHRASE: &str = "correct horse battery staple";

/// Assert the migrated store at `env`'s default project location has
/// exactly the four fixture entries under their sanitized keys.
fn assert_migrated_entries(env: &common::Env) {
    let ls = env
        .command_with_identity()
        .args(["ls", "--json"])
        .output()
        .expect("run ls");
    assert!(ls.status.success());
    let json = json_stdout(&ls);
    let keys: Vec<&str> = json
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry["key"].as_str().expect("key is a string"))
        .collect();
    assert_eq!(keys, ["abc", "easy_as", "multi/line", "unicode"]);

    let get = env
        .command_with_identity()
        .args(["get", "multi/line"])
        .output()
        .expect("run get");
    assert!(get.status.success());
    assert_eq!(get.stdout, b"a\nb");
}

/// Migrating the AES-256-CFB fixture into a new recipients store: `ls`
/// shows the four sanitized keys, `get multi/line`'s bytes match the
/// fixture, and stderr reports the one rename `easy as` needed.
#[test]
fn migrate_symmetric_fixture_into_recipients_store() {
    let env = common::Env::new();
    let passphrase_file = env.path().join("legacy-passphrase.txt");
    std::fs::write(&passphrase_file, SYMMETRIC_PASSPHRASE).expect("write passphrase file");

    let assert = env
        .command()
        .arg("migrate")
        .arg(symmetric_fixture())
        .arg("--passphrase-file")
        .arg(&passphrase_file)
        .args(["--recipient", SSH_PUB.trim(), "--no-self"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("renamed \"easy as\" -> easy_as"),
        "stderr: {stderr}"
    );

    assert_migrated_entries(&env);
}

/// The `--json` shape (appendix 5.1): `ok`, `path`, `entries`, `renamed`
/// (with the one `easy as` -> `easy_as` rename), and `legacy_recipients`.
#[test]
fn migrate_json_output_shape() {
    let env = common::Env::new();
    let passphrase_file = env.path().join("legacy-passphrase.txt");
    std::fs::write(&passphrase_file, SYMMETRIC_PASSPHRASE).expect("write passphrase file");

    let output = env
        .command()
        .arg("--json")
        .arg("migrate")
        .arg(symmetric_fixture())
        .arg("--passphrase-file")
        .arg(&passphrase_file)
        .args(["--recipient", SSH_PUB.trim(), "--no-self"])
        .output()
        .expect("run migrate");
    assert!(output.status.success());
    let json = json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["entries"], 4);
    let renamed = json["renamed"].as_array().expect("renamed is an array");
    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0]["from"], "easy as");
    assert_eq!(renamed[0]["to"], "easy_as");
    assert!(json["legacy_recipients"].is_array());
}

/// Migrating into a passphrase store with two distinct passphrases: the
/// legacy one from `--passphrase-file`, the new one from
/// `--new-passphrase-file`. The migrated store opens only with the new
/// passphrase.
#[test]
fn migrate_into_passphrase_store_with_two_distinct_passphrases() {
    let env = common::Env::new();
    let legacy_passphrase_file = env.path().join("legacy-passphrase.txt");
    std::fs::write(&legacy_passphrase_file, SYMMETRIC_PASSPHRASE).expect("write legacy passphrase");
    let new_passphrase_file = env.path().join("new-passphrase.txt");
    std::fs::write(&new_passphrase_file, "a different passphrase entirely")
        .expect("write new passphrase");

    env.command()
        .arg("migrate")
        .arg(symmetric_fixture())
        .arg("--passphrase-file")
        .arg(&legacy_passphrase_file)
        .arg("--passphrase")
        .arg("--new-passphrase-file")
        .arg(&new_passphrase_file)
        .assert()
        .success();

    // Opens with the new passphrase.
    let unlocked = env
        .command()
        .args(["info", "--json", "--passphrase-file"])
        .arg(&new_passphrase_file)
        .output()
        .expect("run info");
    assert!(unlocked.status.success());
    let json = json_stdout(&unlocked);
    assert_eq!(json["kind"], "passphrase");
    assert_eq!(json["locked"], false);

    // Does not open with the legacy passphrase.
    let locked = env
        .command()
        .args(["info", "--json", "--passphrase-file"])
        .arg(&legacy_passphrase_file)
        .output()
        .expect("run info");
    assert!(locked.status.success());
    let json = json_stdout(&locked);
    assert_eq!(json["locked"], true);
}

/// A target store that already exists: exit 8, and `SOURCE` is never
/// touched (its hash is identical before and after).
#[test]
fn migrate_target_exists_exits_8_and_source_is_untouched() {
    let env = common::Env::new();
    env.init_store();

    let source = symmetric_fixture();
    let before = sha256_of(&source);

    env.command()
        .arg("migrate")
        .arg(&source)
        .args(["--recipient", SSH_PUB.trim(), "--no-self"])
        .assert()
        .failure()
        .code(8);

    let after = sha256_of(&source);
    assert_eq!(
        before, after,
        "the legacy source file must never be modified"
    );
}

/// A source file that is not a legacy v0.4 envelope: exit 1, `not a v0.4
/// store`.
#[test]
fn migrate_not_a_legacy_file_exits_1() {
    let env = common::Env::new();
    let not_legacy = env.path().join("not-legacy.json");
    std::fs::write(&not_legacy, br#"{"hello": "world"}"#).expect("write non-legacy file");

    let assert = env
        .command()
        .arg("migrate")
        .arg(&not_legacy)
        .args(["--recipient", SSH_PUB.trim(), "--no-self"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("not a v0.4 store"), "stderr: {stderr}");
}

/// Opening a legacy v0.4 file with any command other than `migrate`
/// exits 7, the `migrate` hint code (3.6, verified here per step 3.8).
#[test]
fn opening_a_legacy_file_with_another_command_exits_7() {
    let env = common::Env::new();
    env.command()
        .arg("--store")
        .arg(symmetric_fixture())
        .arg("info")
        .assert()
        .failure()
        .code(7);
}

/// `true` if a `gpg` binary usable for the test is on `PATH` (mirrors
/// `crates/trousseau/tests/legacy.rs`'s `gpg_available`).
#[cfg(unix)]
fn gpg_available() -> bool {
    Command::new("gpg")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Migrating the `OpenPGP` fixture: `gpg` decrypts it under an isolated,
/// throwaway `GNUPGHOME` seeded only with the fixture's test key. Unix
/// only (like `crates/trousseau/tests/legacy.rs`'s equivalent: GitHub's
/// Windows runners ship an MSYS `gpg` that rejects a native Windows
/// `GNUPGHOME` path); self-skips when `gpg` is not on `PATH`.
#[cfg(unix)]
#[test]
#[allow(clippy::print_stdout)]
fn migrate_openpgp_fixture_with_gnupg_home() {
    if !gpg_available() {
        println!("skipped: gpg not found");
        return;
    }

    let gnupg_home = tempfile::tempdir().expect("temp GNUPGHOME");
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(gnupg_home.path(), fs::Permissions::from_mode(0o700))
            .expect("chmod 0700 on GNUPGHOME");
    }

    let import = Command::new("gpg")
        .env("GNUPGHOME", gnupg_home.path())
        .args(["--batch", "--import"])
        .arg(test_key_sec())
        .output()
        .expect("gpg --import runs");
    assert!(
        import.status.success(),
        "gpg --import failed: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let env = common::Env::new();
    let assert = env
        .command()
        .arg("migrate")
        .arg(asymmetric_fixture())
        .arg("--gnupg-home")
        .arg(gnupg_home.path())
        .args(["--recipient", SSH_PUB.trim(), "--no-self"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("renamed \"easy as\" -> easy_as"),
        "stderr: {stderr}"
    );

    assert_migrated_entries(&env);

    // Kill the gpg-agent this test started, so the temp GNUPGHOME can be
    // removed cleanly; ignore any failure (the agent may not have
    // started, or may already be gone).
    let _ = Command::new("gpgconf")
        .args(["--homedir"])
        .arg(gnupg_home.path())
        .args(["--kill", "gpg-agent"])
        .output();
}

/// The lowercase hex SHA-256 digest of a file's contents.
fn sha256_of(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let bytes = std::fs::read(path).expect("read file to hash");
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
