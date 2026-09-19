//! Tests for store discovery, locking, classification, and atomic I/O.
//! See `docs/IMPLEMENTATION_PLAN.md` section 3.2, the locking paragraph
//! of 3.5.1, and step 2.4.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use secrecy::SecretString;
use time::macros::datetime;
use trousseau::error::Error;
use trousseau::schema::{Key, Store, StoreKind, Value};
use trousseau::store::{
    Locator, LockMode, PROJECT_STORE_FILENAME, RawStore, Resolved, Seal, Unlock,
    find_project_store, lock, open, read_raw, save, write_atomic,
};

const LEGACY_SYMMETRIC_FIXTURE: &[u8] = include_bytes!("fixtures/legacy/symmetric-v0.4.json");

fn sample_store(kind: StoreKind, recipients: Vec<String>) -> Store {
    let now = datetime!(2026-09-19 00:00:00 UTC);
    let mut store = Store::new(kind, recipients, now);
    store.set(
        Key::parse("database/password").expect("valid key"),
        Value::from_bytes(b"s3cr3t".to_vec()).expect("value within size limit"),
        None,
        None,
        now,
    );
    store
}

// ---- Locator precedence (3.2) ----

#[test]
fn locator_explicit_wins_over_everything() {
    let dir = tempfile::tempdir().expect("temp dir");
    let personal = dir.path().join("personal.trousseau");
    let explicit = dir.path().join("explicit.trousseau");
    let env = dir.path().join("env.trousseau");
    // A project store exists too, to prove explicit still wins over it.
    std::fs::write(dir.path().join(PROJECT_STORE_FILENAME), b"").expect("write project store");

    let locator = Locator {
        explicit: Some(explicit.clone()),
        env: Some(env),
        global: true,
        cwd: dir.path(),
        personal: &personal,
    };
    assert_eq!(locator.resolve(), Resolved::Explicit(explicit.clone()));
    assert_eq!(locator.resolve_for_init(), Resolved::Explicit(explicit));
}

#[test]
fn locator_env_wins_when_no_explicit() {
    let dir = tempfile::tempdir().expect("temp dir");
    let personal = dir.path().join("personal.trousseau");
    let env = dir.path().join("env.trousseau");
    std::fs::write(dir.path().join(PROJECT_STORE_FILENAME), b"").expect("write project store");

    let locator = Locator {
        explicit: None,
        env: Some(env.clone()),
        global: true,
        cwd: dir.path(),
        personal: &personal,
    };
    assert_eq!(locator.resolve(), Resolved::Explicit(env.clone()));
    assert_eq!(locator.resolve_for_init(), Resolved::Explicit(env));
}

#[test]
fn locator_global_wins_when_no_explicit_or_env() {
    let dir = tempfile::tempdir().expect("temp dir");
    let personal = dir.path().join("personal.trousseau");
    std::fs::write(dir.path().join(PROJECT_STORE_FILENAME), b"").expect("write project store");

    let locator = Locator {
        explicit: None,
        env: None,
        global: true,
        cwd: dir.path(),
        personal: &personal,
    };
    assert_eq!(locator.resolve(), Resolved::Personal(personal.clone()));
    assert_eq!(locator.resolve_for_init(), Resolved::Personal(personal));
}

#[test]
fn locator_finds_nearest_project_store_walking_up() {
    let dir = tempfile::tempdir().expect("temp dir");
    let personal = dir.path().join("personal.trousseau");
    let project = dir.path().join(PROJECT_STORE_FILENAME);
    std::fs::write(&project, b"").expect("write project store");
    let nested = dir.path().join("a/b/c");
    std::fs::create_dir_all(&nested).expect("create nested dirs");

    let locator = Locator {
        explicit: None,
        env: None,
        global: false,
        cwd: &nested,
        personal: &personal,
    };
    assert_eq!(locator.resolve(), Resolved::Project(project));
}

#[test]
fn locator_falls_back_to_personal_when_no_project_store() {
    let dir = tempfile::tempdir().expect("temp dir");
    let personal = dir.path().join("personal.trousseau");

    let locator = Locator {
        explicit: None,
        env: None,
        global: false,
        cwd: dir.path(),
        personal: &personal,
    };
    assert_eq!(locator.resolve(), Resolved::Personal(personal));
}

#[test]
fn resolve_for_init_uses_cwd_join_without_walking_up() {
    let dir = tempfile::tempdir().expect("temp dir");
    let personal = dir.path().join("personal.trousseau");
    // A project store exists in an ancestor; `resolve_for_init` must
    // ignore it and never walk up, unlike `resolve`.
    std::fs::write(dir.path().join(PROJECT_STORE_FILENAME), b"")
        .expect("write ancestor project store");
    let nested = dir.path().join("nested");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let locator = Locator {
        explicit: None,
        env: None,
        global: false,
        cwd: &nested,
        personal: &personal,
    };
    assert_eq!(
        locator.resolve_for_init(),
        Resolved::Project(nested.join(PROJECT_STORE_FILENAME))
    );
    assert_eq!(
        locator.resolve(),
        Resolved::Project(dir.path().join(PROJECT_STORE_FILENAME)),
        "resolve() walking up must still find the ancestor's store"
    );
}

// ---- find_project_store ----

#[test]
fn find_project_store_from_nested_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    let project = dir.path().join(PROJECT_STORE_FILENAME);
    std::fs::write(&project, b"").expect("write project store");
    let nested = dir.path().join("a/b/c");
    std::fs::create_dir_all(&nested).expect("create nested dirs");

    assert_eq!(find_project_store(&nested), Some(project));
}

#[test]
fn find_project_store_none_at_root_of_temp_dir() {
    let dir = tempfile::tempdir().expect("temp dir");
    assert_eq!(find_project_store(dir.path()), None);
}

// ---- write_atomic ----

#[test]
fn write_atomic_leaves_no_temp_file_behind_on_success() {
    let dir = tempfile::tempdir().expect("temp dir");
    let target = dir.path().join("store.trousseau");
    write_atomic(&target, b"hello", 0o600).expect("write succeeds");

    assert_eq!(std::fs::read(&target).expect("read target"), b"hello");
    let leftover: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".trousseau-")
        })
        .collect();
    assert!(
        leftover.is_empty(),
        "no .trousseau-* temp file must remain: {leftover:?}"
    );
}

#[cfg(unix)]
#[test]
fn write_atomic_mode_is_0600_after_write() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().expect("temp dir");
    let target = dir.path().join("store.trousseau");
    write_atomic(&target, b"hello", 0o600).expect("write succeeds");

    let mode = std::fs::metadata(&target)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[cfg(unix)]
#[test]
fn write_atomic_failure_leaves_original_file_untouched() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().expect("temp dir");
    let target = dir.path().join("store.trousseau");
    std::fs::write(&target, b"original").expect("seed original file");

    // Read-only (no write bit): blocks creating a new temp file in this
    // directory and therefore the rename that would replace the target,
    // the same permission bit a successful `persist` depends on.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500))
        .expect("make directory read-only");

    let result = write_atomic(&target, b"new", 0o600);

    // Restore write permission so the temp directory can be cleaned up.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))
        .expect("restore directory permissions");

    assert!(
        result.is_err(),
        "write must fail against a read-only directory"
    );
    assert_eq!(
        std::fs::read(&target).expect("read target"),
        b"original",
        "the original file must be untouched by a failed write"
    );
}

// ---- lock ----

#[test]
fn lock_two_shared_guards_coexist() {
    let dir = tempfile::tempdir().expect("temp dir");
    let store_path = dir.path().join("store.trousseau");
    let lock_dir = dir.path().join("locks");

    let first = lock(
        &store_path,
        &lock_dir,
        LockMode::Shared,
        Duration::from_millis(200),
    )
    .expect("first shared lock succeeds");
    let second = lock(
        &store_path,
        &lock_dir,
        LockMode::Shared,
        Duration::from_millis(200),
    )
    .expect("second shared lock succeeds while the first is held");

    drop(first);
    drop(second);
}

#[test]
fn lock_exclusive_times_out_with_existing_shared_then_succeeds_after_drop() {
    let dir = tempfile::tempdir().expect("temp dir");
    let store_path = dir.path().join("store.trousseau");
    let lock_dir = dir.path().join("locks");

    let shared = lock(
        &store_path,
        &lock_dir,
        LockMode::Shared,
        Duration::from_millis(200),
    )
    .expect("shared lock succeeds");

    let err = lock(
        &store_path,
        &lock_dir,
        LockMode::Exclusive,
        Duration::from_millis(200),
    )
    .expect_err("exclusive lock must time out while a shared lock is held");
    assert!(matches!(err, Error::LockTimeout), "got {err:?}");

    drop(shared);

    let exclusive = lock(
        &store_path,
        &lock_dir,
        LockMode::Exclusive,
        Duration::from_millis(200),
    )
    .expect("exclusive lock succeeds once the shared lock is dropped");
    assert_eq!(exclusive.mode(), LockMode::Exclusive);
}

// ---- read_raw classification ----

#[test]
fn read_raw_classifies_age_file_as_current() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    let identity = age::x25519::Identity::generate();
    let recipient: Box<dyn age::Recipient + Send> = Box::new(identity.to_public());
    let sealed =
        trousseau::envelope::seal_to_recipients(b"payload", &[recipient]).expect("seal succeeds");
    std::fs::write(&path, &sealed).expect("write store");

    match read_raw(&path).expect("classification succeeds") {
        RawStore::Current(bytes) => assert_eq!(bytes, sealed),
        RawStore::Legacy(_) => panic!("an age file must classify as current"),
    }
}

#[test]
fn read_raw_classifies_legacy_fixture_as_legacy() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.json");
    std::fs::write(&path, LEGACY_SYMMETRIC_FIXTURE).expect("write fixture");

    match read_raw(&path).expect("classification succeeds") {
        RawStore::Legacy(bytes) => assert_eq!(bytes, LEGACY_SYMMETRIC_FIXTURE),
        RawStore::Current(_) => panic!("the legacy fixture must classify as legacy"),
    }
}

#[test]
fn read_raw_json_missing_data_is_invalid_store() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.json");
    std::fs::write(&path, br#"{"crypto_type":0,"crypto_algorithm":1}"#).expect("write json");

    let err = read_raw(&path).expect_err("a store missing _data must not classify as legacy");
    assert!(matches!(err, Error::InvalidStore { .. }), "got {err:?}");
}

#[test]
fn read_raw_empty_file_is_invalid_store() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    std::fs::write(&path, b"").expect("write empty file");

    let err = read_raw(&path).expect_err("an empty file is neither current nor legacy");
    assert!(matches!(err, Error::InvalidStore { .. }), "got {err:?}");
}

#[test]
fn read_raw_random_binary_is_invalid_store() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    std::fs::write(&path, [0xff_u8, 0x00, 0x10, 0x42, 0xde, 0xad, 0xbe, 0xef])
        .expect("write binary");

    let err = read_raw(&path).expect_err("random binary is neither current nor legacy");
    assert!(matches!(err, Error::InvalidStore { .. }), "got {err:?}");
}

#[test]
fn read_raw_missing_file_is_store_not_found() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("does-not-exist.trousseau");

    let err = read_raw(&path).expect_err("a missing file must not classify");
    assert!(matches!(err, Error::StoreNotFound { .. }), "got {err:?}");
}

// ---- open / save round trip ----

#[test]
fn open_save_round_trip_with_identities() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    let identity = age::x25519::Identity::generate();
    let store = sample_store(
        StoreKind::Recipients,
        vec![identity.to_public().to_string()],
    );

    let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![Box::new(identity.to_public())];
    save(&path, &store, Seal::Recipients(&recipients)).expect("save succeeds");

    let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity)];
    let opened = open(&path, Unlock::Identities(&identities)).expect("open succeeds");
    assert_eq!(opened, store);
}

#[test]
fn open_save_round_trip_with_passphrase() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    let passphrase = SecretString::from("correct horse battery staple".to_owned());
    let store = sample_store(StoreKind::Passphrase, Vec::new());

    save(&path, &store, Seal::Passphrase(&passphrase)).expect("save succeeds");
    let opened = open(&path, Unlock::Passphrase(&passphrase)).expect("open succeeds");
    assert_eq!(opened, store);
}

#[test]
fn save_produces_armor_header() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    let passphrase = SecretString::from("correct horse battery staple".to_owned());
    let store = sample_store(StoreKind::Passphrase, Vec::new());

    save(&path, &store, Seal::Passphrase(&passphrase)).expect("save succeeds");
    let bytes = std::fs::read(&path).expect("read saved store");
    assert!(bytes.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));
}

#[test]
fn save_twice_produces_different_ciphertexts_for_the_same_store() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.trousseau");
    let passphrase = SecretString::from("correct horse battery staple".to_owned());
    let store = sample_store(StoreKind::Passphrase, Vec::new());

    save(&path, &store, Seal::Passphrase(&passphrase)).expect("first save succeeds");
    let first = std::fs::read(&path).expect("read first save");
    save(&path, &store, Seal::Passphrase(&passphrase)).expect("second save succeeds");
    let second = std::fs::read(&path).expect("read second save");

    assert_ne!(first, second, "each save must use a fresh file key");
}

#[test]
fn open_legacy_store_is_legacy_store_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("store.json");
    std::fs::write(&path, LEGACY_SYMMETRIC_FIXTURE).expect("write fixture");

    let err =
        open(&path, Unlock::Identities(&[])).expect_err("a legacy store cannot be opened directly");
    assert!(matches!(err, Error::LegacyStore { .. }), "got {err:?}");
}
