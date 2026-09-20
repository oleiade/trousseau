//! Tests for the legacy v0.4 reader. See `docs/IMPLEMENTATION_PLAN.md`
//! step 2.5 and `crates/trousseau/tests/fixtures/legacy/README.md`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::process::Command;

use secrecy::SecretString;
use time::macros::datetime;
use trousseau::error::Error;
use trousseau::legacy::{
    GpgOptions, LegacyAlgorithm, LegacyStore, convert, decrypt_aes, decrypt_gpg, parse_envelope,
};
use trousseau::schema::{StoreKind, Value};

const SYMMETRIC_FIXTURE: &[u8] = include_bytes!("fixtures/legacy/symmetric-v0.4.json");
const ASYMMETRIC_FIXTURE: &[u8] = include_bytes!("fixtures/legacy/asymmetric-v0.4.json");
const EXPECTED_JSON: &str = include_str!("fixtures/legacy/expected.json");
#[cfg(unix)]
const TEST_KEY_SEC: &[u8] = include_bytes!("fixtures/legacy/test-key.sec.asc");
const SYMMETRIC_PASSPHRASE: &str = "correct horse battery staple";

/// Parse `expected.json` (a flat string map) into the same shape
/// `LegacyStore::data` uses, for comparison.
fn expected_data() -> BTreeMap<String, Value> {
    let raw: BTreeMap<String, String> =
        serde_json::from_str(EXPECTED_JSON).expect("expected.json parses");
    raw.into_iter()
        .map(|(k, v)| (k, Value::from_bytes(v.into_bytes()).expect("value fits")))
        .collect()
}

#[test]
fn parse_envelope_detects_both_fixtures() {
    let symmetric = parse_envelope(SYMMETRIC_FIXTURE).expect("symmetric fixture parses");
    assert_eq!(symmetric.algorithm, LegacyAlgorithm::Aes256Cfb);

    let asymmetric = parse_envelope(ASYMMETRIC_FIXTURE).expect("asymmetric fixture parses");
    assert_eq!(asymmetric.algorithm, LegacyAlgorithm::OpenPgp);
}

#[test]
fn parse_envelope_rejects_a_schema_1_store() {
    let schema_1 = br#"{
  "schema": 1,
  "kind": "passphrase",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T09:41:00Z",
  "recipients": [],
  "entries": {}
}"#;
    assert!(parse_envelope(schema_1).is_err());
}

#[test]
fn parse_envelope_rejects_garbage() {
    assert!(parse_envelope(b"not json at all").is_err());
    assert!(parse_envelope(b"{}").is_err());
}

#[test]
fn decrypt_aes_matches_expected() {
    let envelope = parse_envelope(SYMMETRIC_FIXTURE).expect("parses");
    let passphrase = SecretString::from(SYMMETRIC_PASSPHRASE.to_owned());
    let store = decrypt_aes(&envelope, &passphrase).expect("decrypts");
    assert_eq!(store.data, expected_data());
}

#[test]
fn decrypt_aes_wrong_passphrase_is_unlock_error() {
    let envelope = parse_envelope(SYMMETRIC_FIXTURE).expect("parses");
    let passphrase = SecretString::from("definitely wrong".to_owned());
    let err = decrypt_aes(&envelope, &passphrase).expect_err("wrong passphrase fails");
    assert!(matches!(err, Error::Unlock { .. }));
}

/// `true` if a `gpg` binary usable for the test is on `PATH`.
#[cfg(unix)]
fn gpg_available() -> bool {
    Command::new("gpg")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

// GitHub's Windows runners ship an MSYS gpg that rejects a native Windows
// GNUPGHOME path, so this test runs on Unix only, as the plan intended.
#[cfg(unix)]
#[test]
#[allow(clippy::print_stdout)]
fn decrypt_gpg_matches_expected() {
    if !gpg_available() {
        println!("skipped: gpg not found");
        return;
    }

    let home = tempfile::tempdir().expect("temp GNUPGHOME");
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(home.path(), fs::Permissions::from_mode(0o700))
            .expect("chmod 0700 on GNUPGHOME");
    }

    let import_key_path = home.path().join("test-key.sec.asc");
    std::fs::write(&import_key_path, TEST_KEY_SEC).expect("write throwaway key");
    let import = Command::new("gpg")
        .env("GNUPGHOME", home.path())
        .args(["--batch", "--import"])
        .arg(&import_key_path)
        .output()
        .expect("gpg --import runs");
    assert!(
        import.status.success(),
        "gpg --import failed: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let envelope = parse_envelope(ASYMMETRIC_FIXTURE).expect("parses");
    let opts = GpgOptions {
        binary: "gpg".into(),
        gnupg_home: Some(home.path().to_path_buf()),
    };
    let store = decrypt_gpg(&envelope, &opts).expect("decrypts");
    assert_eq!(store.data, expected_data());

    // Kill the gpg-agent this test started, so the temp GNUPGHOME can be
    // removed cleanly; ignore any failure (the agent may not have
    // started, or may already be gone).
    let _ = Command::new("gpgconf")
        .args(["--homedir"])
        .arg(home.path())
        .args(["--kill", "gpg-agent"])
        .output();
}

const fn now() -> time::OffsetDateTime {
    datetime!(2026-09-19 12:00:00 UTC)
}

fn legacy_store(entries: &[(&str, &str)]) -> LegacyStore {
    let data = entries
        .iter()
        .map(|(k, v)| {
            (
                (*k).to_owned(),
                Value::from_bytes(v.as_bytes().to_vec()).expect("value fits"),
            )
        })
        .collect();
    LegacyStore {
        version: Some("0.4.1".to_owned()),
        recipients: vec!["4B7D890".to_owned()],
        data,
    }
}

#[test]
fn convert_sanitizes_keys_and_carries_recipients() {
    let legacy = legacy_store(&[
        ("abc", "123"),
        ("easy as", "do re mi"),
        ("multi/line", "a\nb"),
        ("unicode", "héllo wörld"),
    ]);
    let conversion = convert(legacy, StoreKind::Passphrase, Vec::new(), now());

    let keys: Vec<&str> = conversion
        .store
        .entries
        .keys()
        .map(trousseau::schema::Key::as_str)
        .collect();
    assert_eq!(keys, ["abc", "easy_as", "multi/line", "unicode"]);
    assert_eq!(conversion.legacy_recipients, ["4B7D890"]);

    // Only "easy as" changed.
    assert_eq!(conversion.renamed.len(), 1);
    assert_eq!(conversion.renamed[0].0, "easy as");
    assert_eq!(conversion.renamed[0].1.as_str(), "easy_as");

    conversion
        .store
        .validate()
        .expect("converted store is valid");
    assert_eq!(conversion.store.created_at, now());
    assert_eq!(conversion.store.updated_at, now());
}

#[test]
fn convert_weird_slashes_are_trimmed_and_collapsed() {
    let legacy = legacy_store(&[("//weird//", "value")]);
    let conversion = convert(legacy, StoreKind::Passphrase, Vec::new(), now());
    let keys: Vec<&str> = conversion
        .store
        .entries
        .keys()
        .map(trousseau::schema::Key::as_str)
        .collect();
    assert_eq!(keys, ["weird"]);
    assert_eq!(conversion.renamed[0].0, "//weird//");
    assert_eq!(conversion.renamed[0].1.as_str(), "weird");
}

#[test]
fn convert_empty_key_becomes_migrated_index() {
    let legacy = legacy_store(&[("", "value")]);
    let conversion = convert(legacy, StoreKind::Passphrase, Vec::new(), now());
    let keys: Vec<&str> = conversion
        .store
        .entries
        .keys()
        .map(trousseau::schema::Key::as_str)
        .collect();
    assert_eq!(keys, ["migrated/0"]);
}

#[test]
fn convert_collisions_get_numeric_suffixes() {
    // Both "a b" and "a/b" sanitize to "a_b" and "a/b" respectively... to
    // force an actual collision, use two keys that sanitize to the exact
    // same string: "a!b" and "a?b" both become "a_b".
    let legacy = legacy_store(&[("a!b", "first"), ("a?b", "second")]);
    let conversion = convert(legacy, StoreKind::Passphrase, Vec::new(), now());
    let mut keys: Vec<&str> = conversion
        .store
        .entries
        .keys()
        .map(trousseau::schema::Key::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["a_b", "a_b_2"]);
    conversion
        .store
        .validate()
        .expect("converted store is valid");
}
