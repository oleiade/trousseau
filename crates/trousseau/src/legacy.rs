//! The legacy v0.4 store reader.
//!
//! This module implements `docs/IMPLEMENTATION_PLAN.md` section 3.7: the
//! legacy envelope (3.7.1), its two payload formats (AES-256-CFB, 3.7.2,
//! and OpenPGP via a spawned `gpg`, 3.7.3), the shared inner document
//! (3.7.4), and the key-sanitizing conversion into a current
//! [`Store`](crate::schema::Store) (3.5.15). It is the only place in the
//! crate that spawns a process.
//!
//! Nothing in this module writes to, or deletes, the legacy source file:
//! callers decide what to do with the result.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use cfb_mode::cipher::KeyIvInit as _;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use time::OffsetDateTime;
use zeroize::Zeroizing;

use crate::error::Error;
use crate::schema::{Key, Store, StoreKind, Value};

/// The length, in bytes, of the salt prefix in an AES-256-CFB payload
/// (3.7.2).
const AES_SALT_LEN: usize = 16;

/// The length, in bytes, of the IV that follows the salt in an
/// AES-256-CFB payload (3.7.2).
const AES_IV_LEN: usize = 16;

/// The derived key length, in bytes, for AES-256 (3.7.2).
const AES_KEY_LEN: usize = 32;

/// scrypt `log2(N)` for the legacy KDF (3.7.2): `N = 65536`.
const SCRYPT_LOG_N: u8 = 16;

/// scrypt block size `r` for the legacy KDF (3.7.2).
const SCRYPT_R: u32 = 16;

/// scrypt parallelism `p` for the legacy KDF (3.7.2).
const SCRYPT_P: u32 = 1;

/// AES-256 in full-block CFB (CFB-128) mode, matching the legacy Go
/// writer's cipher (3.7.2).
type Aes256CfbDecryptor = cfb_mode::Decryptor<aes::Aes256>;

/// Which cipher protects a legacy envelope's payload (3.7.1).
///
/// The envelope's `crypto_algorithm` field decides this; the sibling
/// `crypto_type` field is informational only and is not used here,
/// because the Go implementation's defaults code could leave it at `0`
/// even for an `OpenPGP` store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyAlgorithm {
    /// `crypto_algorithm: 1`: symmetric AES-256-CFB, keyed by scrypt
    /// over a passphrase (3.7.2).
    Aes256Cfb,
    /// `crypto_algorithm: 0`: `OpenPGP`, decrypted by spawning `gpg`
    /// (3.7.3).
    OpenPgp,
}

impl fmt::Display for LegacyAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Aes256Cfb => "AES-256-CFB",
            Self::OpenPgp => "OpenPGP",
        })
    }
}

/// A parsed legacy v0.4 envelope (3.7.1): which algorithm protects it,
/// and the still-encrypted payload bytes (the decoded `_data` field).
#[derive(Clone)]
pub struct LegacyEnvelope {
    /// Which cipher `data` is encrypted with.
    pub algorithm: LegacyAlgorithm,
    /// The decoded ciphertext: for [`LegacyAlgorithm::Aes256Cfb`],
    /// `salt(16) || iv(16) || ciphertext`; for
    /// [`LegacyAlgorithm::OpenPgp`], an ASCII-armored PGP message.
    pub data: Vec<u8>,
}

impl fmt::Debug for LegacyEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LegacyEnvelope")
            .field("algorithm", &self.algorithm)
            .field("data", &format_args!("<{} bytes>", self.data.len()))
            .finish()
    }
}

/// The three fields a legacy envelope must carry, exactly as the Go
/// writer's `encoding/json` produces them.
#[derive(Deserialize)]
struct RawEnvelope {
    /// Informational only (3.7.1): kept so a legacy file missing it is
    /// rejected as not-legacy, but never consulted for the algorithm
    /// decision.
    #[serde(rename = "crypto_type")]
    _crypto_type: u8,
    crypto_algorithm: u8,
    #[serde(rename = "_data")]
    data: String,
}

/// Parse and decode a legacy v0.4 envelope (3.7.1).
///
/// `bytes` must be a JSON object with `crypto_type`, `crypto_algorithm`
/// and `_data` (standard base64); anything else, including a schema 1
/// store or arbitrary garbage, is rejected.
///
/// # Errors
///
/// Returns [`Error::InvalidStore`] if `bytes` is not well-formed JSON
/// with the three required fields, if `crypto_algorithm` is neither `0`
/// nor `1`, or if `_data` is not valid standard base64.
pub fn parse_envelope(bytes: &[u8]) -> Result<LegacyEnvelope, Error> {
    let raw: RawEnvelope = serde_json::from_slice(bytes).map_err(|_err| Error::InvalidStore {
        reason: "not a legacy v0.4 store".to_owned(),
    })?;

    let algorithm = match raw.crypto_algorithm {
        1 => LegacyAlgorithm::Aes256Cfb,
        0 => LegacyAlgorithm::OpenPgp,
        other => {
            return Err(Error::InvalidStore {
                reason: format!("unsupported legacy crypto_algorithm {other}"),
            });
        }
    };

    let data = STANDARD
        .decode(raw.data.as_bytes())
        .map_err(|_err| Error::InvalidStore {
            reason: "legacy _data is not valid base64".to_owned(),
        })?;

    Ok(LegacyEnvelope { algorithm, data })
}

/// A legacy store's decrypted inner document (3.7.4): optional
/// metadata, and the flat key/value map.
///
/// `data`'s values are wrapped as [`Value`] so [`convert`] can hand them
/// straight to [`Store::set`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyStore {
    /// The legacy writer's version string, if present.
    pub version: Option<String>,
    /// Legacy recipients (PGP key ids), printed for information only;
    /// [`convert`] does not carry them into the new store's recipients.
    pub recipients: Vec<String>,
    /// The flat key/value map, keyed by the legacy (unsanitized) key.
    pub data: BTreeMap<String, Value>,
}

/// The `metadata` object inside a legacy inner document (3.7.4). Every
/// field is optional; fields this build does not know about are
/// ignored, matching the legacy writer, which was not strict either.
#[derive(Deserialize, Default)]
struct LegacyMetadata {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    recipients: Vec<String>,
}

/// The legacy inner document (3.7.4): `metadata` and a flat
/// string-to-string `data` map. Unknown top-level fields are ignored.
#[derive(Deserialize)]
struct LegacyDocument {
    #[serde(default)]
    metadata: LegacyMetadata,
    #[serde(default)]
    data: BTreeMap<String, String>,
}

/// Parse the shared inner document (3.7.4) from already-decrypted
/// bytes.
///
/// # Errors
///
/// Returns [`Error::Legacy`] if `bytes` is not well-formed JSON matching
/// 3.7.4, or [`Error::TooLarge`] if any single value exceeds
/// [`crate::schema::MAX_VALUE_BYTES`].
fn parse_inner_document(bytes: &[u8]) -> Result<LegacyStore, Error> {
    let document: LegacyDocument = serde_json::from_slice(bytes).map_err(|_err| Error::Legacy {
        reason: "not a valid legacy v0.4 document".to_owned(),
    })?;
    let mut data = BTreeMap::new();
    for (key, value) in document.data {
        data.insert(key, Value::from_bytes(value.into_bytes())?);
    }
    Ok(LegacyStore {
        version: document.metadata.version,
        recipients: document.metadata.recipients,
        data,
    })
}

/// Decrypt an AES-256-CFB legacy envelope (3.7.2).
///
/// `env.data` must be `salt(16) || iv(16) || ciphertext`. The key is
/// `scrypt(passphrase, salt, log_n=16, r=16, p=1, dkLen=32)`; the
/// derived key and the decrypted buffer are zeroized on drop.
///
/// A wrong passphrase decrypts to garbage that fails to parse as the
/// inner document (3.7.4); that case, and a payload too short to even
/// contain the salt and IV, are both reported the same way, so nothing
/// about *why* unlocking failed leaks.
///
/// # Errors
///
/// Returns [`Error::Legacy`] if `env` is not an AES-256-CFB envelope,
/// and [`Error::Unlock`] if the passphrase is wrong or the payload is
/// corrupted.
pub fn decrypt_aes(env: &LegacyEnvelope, passphrase: &SecretString) -> Result<LegacyStore, Error> {
    if env.algorithm != LegacyAlgorithm::Aes256Cfb {
        return Err(Error::Legacy {
            reason: format!("legacy envelope uses {}, not AES-256-CFB", env.algorithm),
        });
    }
    if env.data.len() < AES_SALT_LEN + AES_IV_LEN {
        return Err(Error::Unlock {
            reason: "wrong passphrase or corrupted store".to_owned(),
        });
    }
    let (salt, rest) = env.data.split_at(AES_SALT_LEN);
    let (iv, ciphertext) = rest.split_at(AES_IV_LEN);

    let mut key = Zeroizing::new([0u8; AES_KEY_LEN]);
    let params =
        scrypt::Params::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P).map_err(|err| Error::Legacy {
            reason: format!("invalid scrypt parameters: {err}"),
        })?;
    scrypt::scrypt(
        passphrase.expose_secret().as_bytes(),
        salt,
        &params,
        key.as_mut_slice(),
    )
    .map_err(|_err| Error::Legacy {
        reason: "scrypt key derivation failed".to_owned(),
    })?;

    let mut plaintext = Zeroizing::new(ciphertext.to_vec());
    let decryptor =
        Aes256CfbDecryptor::new_from_slices(key.as_slice(), iv).map_err(|_err| Error::Legacy {
            reason: "invalid AES key or IV length".to_owned(),
        })?;
    decryptor.decrypt(plaintext.as_mut_slice());

    parse_inner_document(&plaintext).map_err(|_err| Error::Unlock {
        reason: "wrong passphrase or corrupted store".to_owned(),
    })
}

/// Where to find `gpg` and, optionally, an isolated `GNUPGHOME` for
/// [`decrypt_gpg`].
#[derive(Clone, Debug)]
pub struct GpgOptions {
    /// The `gpg` binary to spawn (a bare name resolved on `PATH`, or a
    /// full path).
    pub binary: PathBuf,
    /// When `Some`, set as the child process's `GNUPGHOME` environment
    /// variable. When `None`, the child inherits the caller's
    /// environment unchanged (including any `GNUPGHOME` already set).
    pub gnupg_home: Option<PathBuf>,
}

/// Decrypt an `OpenPGP` legacy envelope (3.7.3) by spawning `gpg`.
///
/// Runs `<opts.binary> --batch --quiet --decrypt` with `env.data` (the
/// armored PGP message) piped to stdin, then closes stdin and waits for
/// the child to finish. No passphrase is ever passed to the child;
/// `gpg-agent`/`pinentry` handle that out of band. The caller's
/// environment is never cleared, only extended with `GNUPGHOME` when
/// `opts.gnupg_home` is set.
///
/// # Errors
///
/// Returns [`Error::Legacy`] if `env` is not an `OpenPGP` envelope, if the
/// binary cannot be spawned (the reason names the binary and the `io`
/// error, never anything secret), or if `gpg` exits non-zero (the
/// reason is the last non-empty line of its stderr). Returns
/// [`Error::Io`] if writing to the child's stdin or reading its output
/// fails for a reason other than a non-zero exit.
pub fn decrypt_gpg(env: &LegacyEnvelope, opts: &GpgOptions) -> Result<LegacyStore, Error> {
    if env.algorithm != LegacyAlgorithm::OpenPgp {
        return Err(Error::Legacy {
            reason: format!("legacy envelope uses {}, not OpenPGP", env.algorithm),
        });
    }

    let mut command = Command::new(&opts.binary);
    command.args(["--batch", "--quiet", "--decrypt"]);
    if let Some(home) = &opts.gnupg_home {
        command.env("GNUPGHOME", home);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|err| Error::Legacy {
        reason: format!("cannot run {}: {err}", opts.binary.display()),
    })?;

    // Write the armored message and close stdin (by dropping it) before
    // waiting, so `gpg` sees EOF and does not block forever.
    let mut stdin = child.stdin.take().ok_or_else(|| Error::Legacy {
        reason: "cannot open gpg stdin".to_owned(),
    })?;
    stdin.write_all(&env.data).map_err(Error::Io)?;
    drop(stdin);

    let output = child.wait_with_output().map_err(Error::Io)?;
    if !output.status.success() {
        return Err(Error::Legacy {
            reason: last_non_empty_line(&output.stderr),
        });
    }

    parse_inner_document(&output.stdout)
}

/// The last non-empty line of `bytes`, decoded lossily, trimmed. Falls
/// back to a generic message if every line is empty.
fn last_non_empty_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map_or_else(
            || "gpg failed with no diagnostic output".to_owned(),
            |line| line.trim().to_owned(),
        )
}

/// The result of [`convert`]: the migrated store, the keys that had to
/// be renamed during sanitization, and the legacy recipients (carried
/// for information only).
#[derive(Debug)]
pub struct Conversion {
    /// The migrated store, ready to be sealed and saved.
    pub store: Store,
    /// `(original legacy key, sanitized key)` for every legacy key whose
    /// sanitized form differs from the original.
    pub renamed: Vec<(String, Key)>,
    /// The legacy store's recipients (PGP key ids), unchanged, for the
    /// caller to print. They are not otherwise used: the migrated
    /// store's recipients come from `convert`'s own `recipients`
    /// argument.
    pub legacy_recipients: Vec<String>,
}

/// Convert a decrypted legacy store into a current [`Store`] (3.5.15).
///
/// Each legacy key is sanitized: every character outside
/// `[A-Za-z0-9._/-]` becomes `_`, repeated `/` collapse to one, and
/// leading/trailing `/` are trimmed. If the result is empty or still
/// not a valid [`Key`] (for example, a segment that sanitizes down to
/// `.` or `..`), it is replaced by `migrated/<index>`, where `<index>`
/// is the key's position in sorted legacy key order, starting at `0`.
/// A sanitized name that collides with an earlier one gets a `_2`,
/// `_3`, ... suffix. Every entry whose final key differs from its
/// original legacy key is reported in
/// [`Conversion::renamed`](Conversion::renamed).
///
/// The new store is built with [`Store::new`] from `kind`, `recipients`
/// and `now` (also used as every migrated entry's `created_at` and
/// `updated_at`). This function cannot fail: the sanitizer is built to
/// always produce a valid [`Key`] (see the `sanitize_is_always_a_valid_key`
/// proptest in this module's tests), so the returned store is valid by
/// construction.
#[must_use]
pub fn convert(
    legacy: LegacyStore,
    kind: StoreKind,
    recipients: Vec<String>,
    now: OffsetDateTime,
) -> Conversion {
    let mut store = Store::new(kind, recipients, now);
    let mut renamed = Vec::new();
    let mut used = BTreeSet::new();

    for (index, (original, value)) in legacy.data.into_iter().enumerate() {
        let candidate = sanitize(&original);
        let key = key_or_migrated(&candidate, index);
        let key = dedupe(&mut used, key);
        if key.as_str() != original {
            renamed.push((original, key.clone()));
        }
        store.set(key, value, None, None, now);
    }

    Conversion {
        store,
        renamed,
        legacy_recipients: legacy.recipients,
    }
}

/// Replace every character outside `[A-Za-z0-9._/-]` with `_`, collapse
/// repeated `/`, and trim leading/trailing `/` (3.5.15). The result may
/// still fail the [`Key`] grammar (for example, an empty string or a
/// segment starting with `.` or `_`); [`key_or_migrated`] handles that.
fn sanitize(raw: &str) -> String {
    let mut replaced = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/') {
            replaced.push(ch);
        } else {
            replaced.push('_');
        }
    }

    let mut collapsed = String::with_capacity(replaced.len());
    let mut last_was_slash = false;
    for ch in replaced.chars() {
        let is_slash = ch == '/';
        if is_slash && last_was_slash {
            continue;
        }
        collapsed.push(ch);
        last_was_slash = is_slash;
    }

    collapsed.trim_matches('/').to_owned()
}

/// Parse `candidate` as a [`Key`], falling back to `migrated/<n>`
/// (trying `n = index`, then `index + 1`, ...) if it is empty or
/// otherwise invalid.
///
/// This never panics. `migrated/<n>` satisfies the `Key` grammar for
/// every `usize`, so the fallback loop always returns on its first
/// iteration in practice; it is written as a loop, rather than an
/// `unwrap`, so that remains true by construction instead of by
/// assumption.
fn key_or_migrated(candidate: &str, index: usize) -> Key {
    if let Ok(key) = Key::parse(candidate) {
        return key;
    }
    let mut n = index;
    loop {
        if let Ok(key) = Key::parse(&format!("migrated/{n}")) {
            return key;
        }
        n = n.wrapping_add(1);
    }
}

/// If `key` was already produced for an earlier entry, append `_2`,
/// `_3`, ... until an unused name is found.
fn dedupe(used: &mut BTreeSet<String>, key: Key) -> Key {
    if used.insert(key.as_str().to_owned()) {
        return key;
    }
    let mut suffix: u64 = 2;
    loop {
        let candidate = format!("{key}_{suffix}");
        if let Ok(candidate_key) = Key::parse(&candidate)
            && used.insert(candidate)
        {
            return candidate_key;
        }
        suffix += 1;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{Key, key_or_migrated, sanitize};
    use proptest::prelude::*;

    #[test]
    fn sanitize_examples() {
        assert_eq!(sanitize("abc"), "abc");
        assert_eq!(sanitize("easy as"), "easy_as");
        assert_eq!(sanitize("multi/line"), "multi/line");
        assert_eq!(sanitize("//weird//"), "weird");
        assert_eq!(sanitize(""), "");
    }

    proptest! {
        #[test]
        fn sanitize_is_always_a_valid_key(raw in ".{0,64}", index in 0usize..1000) {
            let candidate = sanitize(&raw);
            let key = key_or_migrated(&candidate, index);
            prop_assert!(Key::parse(key.as_str()).is_ok());
        }

        #[test]
        fn migrated_fallback_is_always_valid(index in any::<usize>()) {
            let key = key_or_migrated("", index);
            prop_assert!(Key::parse(key.as_str()).is_ok());
        }
    }
}
