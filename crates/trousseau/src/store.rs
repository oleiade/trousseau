//! Store discovery, locking, and atomic on-disk I/O.
//!
//! This module implements `docs/IMPLEMENTATION_PLAN.md` section 3.2 (store
//! discovery and paths) and the locking paragraph of section 3.5.1: where a
//! store lives, how a store file's bytes are classified (current age
//! envelope, legacy v0.4, or invalid), how a store is opened and saved, and
//! how reads and writes are made atomic and mutually exclusive across
//! processes.

use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use secrecy::SecretString;
use sha2::{Digest, Sha256};

use crate::envelope;
use crate::error::Error;
use crate::schema::Store;

/// The filename of a project store (3.2): `.trousseau` in the current
/// directory or an ancestor.
pub const PROJECT_STORE_FILENAME: &str = ".trousseau";

/// The file mode a saved store is written with on Unix (3.2).
const STORE_FILE_MODE: u32 = 0o600;

/// The armor header every current-format store starts with (3.1.1).
///
/// This mirrors the private constant of the same name in
/// [`crate::envelope`]; it is duplicated here rather than exposed from
/// there so `envelope` does not need a `pub(crate)` item just for this
/// module's classification check.
const ARMOR_HEADER: &[u8] = b"-----BEGIN AGE ENCRYPTED FILE-----";

/// How often [`lock`] polls for the lock to become available.
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// How many times [`write_atomic`] retries `persist` on Windows before
/// giving up (implementation notes, step 2.4).
#[cfg(windows)]
const WINDOWS_PERSIST_RETRIES: u32 = 3;

/// A resolved store path, tagged with which rule of 3.2 produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// `--store PATH` or `TROUSSEAU_STORE` (rules 1 and 2).
    Explicit(PathBuf),
    /// The nearest `.trousseau` file found walking up from the current
    /// directory (rule 4), or, for [`Locator::resolve_for_init`], the
    /// current directory's `.trousseau` regardless of whether it exists.
    Project(PathBuf),
    /// The user's personal store (`--global`, rule 3, or the final
    /// fallback, rule 5).
    Personal(PathBuf),
}

impl Resolved {
    /// The resolved path, regardless of which rule produced it.
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::Explicit(path) | Self::Project(path) | Self::Personal(path) => path,
        }
    }
}

/// Inputs to store resolution (3.2): everything [`Locator::resolve`] and
/// [`Locator::resolve_for_init`] need, gathered from the CLI's flags,
/// environment, and configuration.
#[derive(Debug, Clone)]
pub struct Locator<'a> {
    /// `--store PATH`, if given (rule 1).
    pub explicit: Option<PathBuf>,
    /// `TROUSSEAU_STORE`, if set (rule 2).
    pub env: Option<PathBuf>,
    /// `--global`, if given (rule 3).
    pub global: bool,
    /// The current directory, walked upward for rule 4.
    pub cwd: &'a Path,
    /// The user's personal store path (rules 3 and 5).
    pub personal: &'a Path,
}

impl Locator<'_> {
    /// Resolve a store path per 3.2's order, for every command except
    /// `init`.
    #[must_use]
    pub fn resolve(&self) -> Resolved {
        self.resolve_with(true)
    }

    /// Resolve a store path for `init`: identical to [`Locator::resolve`]
    /// except rule 4 becomes "the current directory's `.trousseau`",
    /// never walking up to an ancestor's store.
    #[must_use]
    pub fn resolve_for_init(&self) -> Resolved {
        self.resolve_with(false)
    }

    /// The shared body of [`Locator::resolve`] and
    /// [`Locator::resolve_for_init`]: rules 1 through 3 and 5 are
    /// identical between them, and `walk_up` picks rule 4's behavior.
    fn resolve_with(&self, walk_up: bool) -> Resolved {
        if let Some(path) = &self.explicit {
            return Resolved::Explicit(path.clone());
        }
        if let Some(path) = &self.env {
            return Resolved::Explicit(path.clone());
        }
        if self.global {
            return Resolved::Personal(self.personal.to_path_buf());
        }
        if walk_up {
            if let Some(path) = find_project_store(self.cwd) {
                return Resolved::Project(path);
            }
            return Resolved::Personal(self.personal.to_path_buf());
        }
        Resolved::Project(self.cwd.join(PROJECT_STORE_FILENAME))
    }
}

/// Walk `start` and its ancestors looking for a [`PROJECT_STORE_FILENAME`]
/// that is a regular file (3.2, rule 4).
///
/// A symlink to a regular file counts: this checks [`std::fs::metadata`]
/// (which follows symlinks), not `symlink_metadata`.
#[must_use]
pub fn find_project_store(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let candidate = dir.join(PROJECT_STORE_FILENAME);
        if fs::metadata(&candidate).is_ok_and(|metadata| metadata.is_file()) {
            return Some(candidate);
        }
    }
    None
}

/// Which kind of advisory lock [`lock`] acquires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    /// A shared (read) lock: any number of readers may hold it at once.
    Shared,
    /// An exclusive (write) lock: only one holder at a time, excluding
    /// every reader too.
    Exclusive,
}

/// A held advisory lock on a store's lock file (3.2, 3.5.1).
///
/// Dropping a `LockGuard` releases the lock. See [`lock`]'s doc comment
/// for why that happens by closing the lock file rather than through an
/// explicit unlock call.
#[derive(Debug)]
pub struct LockGuard {
    // Never read: this field's only purpose is to stay alive and be
    // dropped. The OS-level lock acquired in `lock` is released when
    // this is dropped (closing the file descriptor/handle), not by an
    // explicit unlock call: see `lock`'s doc comment.
    #[allow(dead_code)]
    lock: fd_lock::RwLock<File>,
    mode: LockMode,
}

impl LockGuard {
    /// Which mode this guard holds the lock in.
    #[must_use]
    pub const fn mode(&self) -> LockMode {
        self.mode
    }
}

/// Acquire an advisory lock on the store at `store_path`, polling every
/// 50 ms until `timeout` elapses.
///
/// The lock file lives at
/// `<lock_dir>/<sha256 hex of the store path's canonical form>.lock`
/// (3.2); `lock_dir` is created (mode `0700` on Unix) if it does not
/// exist yet. The canonical form used for hashing follows
/// `std::fs::canonicalize` when the store exists, falling back to the
/// canonicalized parent directory (or the path as given) when it does
/// not, so the lock name stays stable across relative/absolute
/// spellings of the same store path.
///
/// The returned [`LockGuard`] never calls `fd_lock`'s own unlock. The
/// guard `try_read`/`try_write` return borrows the `fd_lock::RwLock` for
/// its own lifetime, so storing that guard alongside the lock in the
/// same long-lived struct is not possible without `unsafe`, which this
/// crate forbids. Instead, the guard is acquired and immediately leaked
/// with [`std::mem::forget`] — safe, and the documented way to keep an
/// `fd-lock` lock held past its guard's lifetime. The OS-level lock
/// (`flock` on Unix, `LockFileEx` on Windows) then stays in effect until
/// the underlying file is closed, which happens when the returned
/// `LockGuard` is dropped. A test in `tests/store_io.rs` proves this:
/// dropping a `LockGuard` lets a subsequent, otherwise-conflicting `lock`
/// call succeed.
///
/// # Errors
///
/// Returns [`Error::LockTimeout`] if the lock is still held by another
/// process when `timeout` elapses, or [`Error::Io`] if the lock
/// directory or file cannot be created or opened.
pub fn lock(
    store_path: &Path,
    lock_dir: &Path,
    mode: LockMode,
    timeout: Duration,
) -> Result<LockGuard, Error> {
    ensure_lock_dir(lock_dir)?;
    let lock_path = lock_dir.join(format!("{}.lock", lock_file_name(store_path)));
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(Error::Io)?;

    let mut rw = fd_lock::RwLock::new(file);
    let deadline = Instant::now() + timeout;
    loop {
        let attempt = match mode {
            LockMode::Shared => rw.try_read().map(std::mem::forget),
            LockMode::Exclusive => rw.try_write().map(std::mem::forget),
        };
        match attempt {
            Ok(()) => return Ok(LockGuard { lock: rw, mode }),
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(Error::LockTimeout);
                }
                std::thread::sleep(LOCK_POLL_INTERVAL);
            }
            Err(err) => return Err(Error::Io(err)),
        }
    }
}

/// Create `lock_dir` if it does not exist yet, setting its permissions to
/// `0700` on Unix (3.2).
fn ensure_lock_dir(lock_dir: &Path) -> Result<(), Error> {
    fs::create_dir_all(lock_dir).map_err(Error::Io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(lock_dir, fs::Permissions::from_mode(0o700)).map_err(Error::Io)?;
    }
    Ok(())
}

/// The path used to name a store's lock file: `std::fs::canonicalize` of
/// `store_path` if the store exists, else the canonicalized parent
/// directory joined with the store's file name, else `store_path` itself
/// unchanged if even that fails (for example, if the parent does not
/// exist either).
///
/// This keeps the lock file name stable across relative/absolute
/// spellings of the same store path without requiring the store to exist
/// yet (`init` locks a store that is about to be created).
fn canonical_lock_target(store_path: &Path) -> PathBuf {
    if let Ok(canonical) = fs::canonicalize(store_path) {
        return canonical;
    }
    if let Some(parent) = store_path.parent()
        && let Ok(canonical_parent) = fs::canonicalize(parent)
    {
        return match store_path.file_name() {
            Some(name) => canonical_parent.join(name),
            None => canonical_parent,
        };
    }
    store_path.to_path_buf()
}

/// The lock file's base name (without `.lock`): the lowercase hex SHA-256
/// digest of [`canonical_lock_target`]'s bytes.
fn lock_file_name(store_path: &Path) -> String {
    let canonical = canonical_lock_target(store_path);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_os_str().as_encoded_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        // Writing to a `String` through `fmt::Write` never fails.
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// The result of classifying a store file's raw bytes. See [`read_raw`]
/// for the classification order.
#[derive(Debug)]
pub enum RawStore {
    /// An ASCII-armored age file: the current store format.
    Current(Vec<u8>),
    /// A legacy v0.4 envelope (3.7.1); needs `trousseau migrate`.
    Legacy(Vec<u8>),
}

/// Read and classify the store file at `path`.
///
/// Classification order: a missing file is [`Error::StoreNotFound`];
/// bytes starting with the age armor header are [`RawStore::Current`];
/// otherwise, bytes that look like a legacy v0.4 envelope (3.7.1) are
/// [`RawStore::Legacy`]; anything else is [`Error::InvalidStore`].
///
/// # Errors
///
/// Returns [`Error::StoreNotFound`] if `path` does not exist,
/// [`Error::InvalidStore`] if its bytes match neither the current nor
/// the legacy format, or [`Error::Io`] for any other I/O failure.
pub fn read_raw(path: &Path) -> Result<RawStore, Error> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Err(Error::StoreNotFound {
                path: path.to_path_buf(),
            });
        }
        Err(err) => return Err(Error::Io(err)),
    };
    if bytes.starts_with(ARMOR_HEADER) {
        return Ok(RawStore::Current(bytes));
    }
    if looks_like_legacy(&bytes) {
        return Ok(RawStore::Legacy(bytes));
    }
    Err(Error::InvalidStore {
        reason: "not an age file or a recognized legacy v0.4 store".to_owned(),
    })
}

/// Legacy v0.4 envelope detection (3.7.1), delegated to
/// [`crate::legacy::parse_envelope`]: `true` if `bytes` is a JSON object
/// with `crypto_type`, `crypto_algorithm` and base64 `_data`. Nothing
/// about the decrypted payload is checked here.
fn looks_like_legacy(bytes: &[u8]) -> bool {
    crate::legacy::parse_envelope(bytes).is_ok()
}

/// How to unlock a store when [`open`]ing it.
pub enum Unlock<'a> {
    /// Try each identity in turn (`age` tries every one in order).
    Identities(&'a [Box<dyn age::Identity>]),
    /// Unlock a passphrase store.
    Passphrase(&'a SecretString),
}

/// Read, classify, decrypt, and parse the store at `path`.
///
/// Equivalent to [`read_raw`] followed by
/// [`envelope::open_with_identities`] or
/// [`envelope::open_with_passphrase`], then [`Store::from_json`].
///
/// # Errors
///
/// Returns [`Error::StoreNotFound`] or [`Error::InvalidStore`] as
/// [`read_raw`] does, [`Error::LegacyStore`] if the store is a legacy
/// v0.4 store, or whatever [`envelope::open_with_identities`],
/// [`envelope::open_with_passphrase`], or [`Store::from_json`] returns.
// `unlock` is a fixed part of this step's public API (an enum whose
// payload is entirely shared references, cheap to move), even though
// each match arm only reads one variant.
#[allow(clippy::needless_pass_by_value)]
pub fn open(path: &Path, unlock: Unlock<'_>) -> Result<Store, Error> {
    match read_raw(path)? {
        RawStore::Legacy(_) => Err(Error::LegacyStore {
            path: path.to_path_buf(),
        }),
        RawStore::Current(bytes) => {
            let plaintext = match unlock {
                Unlock::Identities(identities) => {
                    envelope::open_with_identities(&bytes, identities)?
                }
                Unlock::Passphrase(passphrase) => {
                    envelope::open_with_passphrase(&bytes, passphrase)?
                }
            };
            Store::from_json(&plaintext)
        }
    }
}

/// How to seal a store when [`save`]ing it.
pub enum Seal<'a> {
    /// Seal to a set of age recipients.
    Recipients(&'a [Box<dyn age::Recipient + Send>]),
    /// Seal to a passphrase.
    Passphrase(&'a SecretString),
}

/// Validate, serialize, seal, and atomically write `store` to `path`.
///
/// Every save produces a fresh age file key (3.1.1): sealing the same
/// store twice never produces the same ciphertext.
///
/// # Errors
///
/// Returns whatever [`Store::to_json`] returns for an invalid or
/// oversized store, whatever [`envelope::seal_to_recipients`] or
/// [`envelope::seal_with_passphrase`] returns, or whatever
/// [`write_atomic`] returns.
// `seal` is a fixed part of this step's public API (an enum whose
// payload is entirely shared references, cheap to move), even though
// each match arm only reads one variant.
#[allow(clippy::needless_pass_by_value)]
pub fn save(path: &Path, store: &Store, seal: Seal<'_>) -> Result<(), Error> {
    let plaintext = store.to_json()?;
    let sealed = match seal {
        Seal::Recipients(recipients) => envelope::seal_to_recipients(&plaintext, recipients)?,
        Seal::Passphrase(passphrase) => envelope::seal_with_passphrase(&plaintext, passphrase)?,
    };
    write_atomic(path, &sealed, STORE_FILE_MODE)
}

/// Atomically write `bytes` to `path`, replacing any existing file.
///
/// Writes to a temporary file (prefix `.trousseau-`) created in `path`'s
/// parent directory, sets its permissions to `mode` on Unix before
/// writing, writes and `fsync`s it, then renames it over `path` and
/// `fsync`s the parent directory on Unix. On Windows, `persist` can fail
/// transiently if the target is open elsewhere; this retries up to three
/// times with a 50 ms sleep between attempts.
///
/// `path`'s parent directory must already exist.
///
/// # Errors
///
/// Returns [`Error::Io`] if `path` has no parent directory, or if the
/// temporary file cannot be created, written, or persisted.
pub fn write_atomic(path: &Path, bytes: &[u8], mode: u32) -> Result<(), Error> {
    let parent = path.parent().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            ErrorKind::InvalidInput,
            "store path has no parent directory",
        ))
    })?;

    let mut temp = tempfile::Builder::new()
        .prefix(".trousseau-")
        .tempfile_in(parent)
        .map_err(Error::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        temp.as_file()
            .set_permissions(fs::Permissions::from_mode(mode))
            .map_err(Error::Io)?;
    }
    #[cfg(not(unix))]
    {
        // Windows relies on the user profile's ACLs (3.2); `mode` is
        // unused there.
        let _ = mode;
    }

    temp.write_all(bytes).map_err(Error::Io)?;
    temp.as_file().sync_all().map_err(Error::Io)?;

    persist(temp, path)?;

    #[cfg(unix)]
    {
        File::open(parent)
            .and_then(|dir| dir.sync_all())
            .map_err(Error::Io)?;
    }

    Ok(())
}

/// Rename `temp` over `path`.
///
/// On Windows, retries up to [`WINDOWS_PERSIST_RETRIES`] times with a
/// 50 ms sleep between attempts, since `persist` can fail transiently
/// there if the target is open elsewhere.
fn persist(temp: tempfile::NamedTempFile, path: &Path) -> Result<(), Error> {
    #[cfg(windows)]
    {
        let mut temp = temp;
        for attempt in 0..WINDOWS_PERSIST_RETRIES {
            match temp.persist(path) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    temp = err.file;
                    if attempt + 1 == WINDOWS_PERSIST_RETRIES {
                        return Err(Error::Io(err.error));
                    }
                    std::thread::sleep(LOCK_POLL_INTERVAL);
                }
            }
        }
        // Unreachable: `WINDOWS_PERSIST_RETRIES` is non-zero, so the loop
        // above always returns on its last iteration.
        Err(Error::Io(std::io::Error::other(
            "persist failed after retries",
        )))
    }
    #[cfg(not(windows))]
    {
        temp.persist(path)
            .map(|_file| ())
            .map_err(|err| Error::Io(err.error))
    }
}
