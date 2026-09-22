//! Schema 1: the store's decrypted JSON payload.
//!
//! This module implements everything in `docs/IMPLEMENTATION_PLAN.md`
//! section 3.1: the [`Key`](crate::schema::Key) grammar, the
//! [`Value`](crate::schema::Value) wrapper that redacts secret bytes,
//! [`Entry`](crate::schema::Entry) and [`Store`](crate::schema::Store),
//! their JSON representation, and the environment variable mapping. See
//! `docs/format.md` for the user-facing restatement.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry as MapEntry;
use std::fmt;
use std::str::FromStr;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::OffsetDateTime;

use crate::error::Error;

/// The only schema version this build understands.
pub const SCHEMA_VERSION: u32 = 1;

/// The maximum size, in bytes, of a store's decrypted JSON payload.
pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

/// The maximum size, in bytes, of a single entry's value.
pub const MAX_VALUE_BYTES: usize = 4 * 1024 * 1024;

/// The maximum size, in bytes, of an entry's description.
const MAX_DESCRIPTION_BYTES: usize = 1024;

/// The maximum total length, in bytes, of a [`Key`].
const MAX_KEY_BYTES: usize = 256;

/// Truncate an [`OffsetDateTime`] to whole seconds, as required by the
/// on-disk timestamp format (3.1.2).
///
/// Also used directly by the CLI crate's `edit` and `import` commands,
/// which write entries outside [`Store`]'s own always-bump setters (see
/// their own doc comments for why).
#[must_use]
pub fn truncate_to_seconds(t: OffsetDateTime) -> OffsetDateTime {
    t.replace_nanosecond(0).unwrap_or(t)
}

/// `true` if `s` matches `^[A-Za-z_][A-Za-z0-9_]*$`, the grammar shared by
/// the `env` override field and the `env_prefix` argument (3.1.4).
fn is_valid_env_name(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A validated store key: a `/`-separated path used to address a secret.
///
/// A `Key` can only be built through [`Key::parse`] (or the equivalent
/// [`FromStr`] / [`TryFrom<String>`] impls), which enforces the grammar in
/// `docs/format.md`'s "Keys" section: one or more segments matching
/// `[A-Za-z0-9][A-Za-z0-9._-]*`, no empty, leading, or trailing segments,
/// and a total length of at most 256 bytes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Key(String);

impl Key {
    /// Parse and validate a key string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidKey`] if `s` is empty, exceeds 256 bytes,
    /// starts or ends with `/`, contains an empty segment, or contains a
    /// segment that does not match `[A-Za-z0-9][A-Za-z0-9._-]*`.
    pub fn parse(s: &str) -> Result<Self, Error> {
        if s.is_empty() {
            return Err(Error::InvalidKey {
                input: s.to_owned(),
                reason: "key must not be empty".to_owned(),
            });
        }
        if s.len() > MAX_KEY_BYTES {
            return Err(Error::InvalidKey {
                input: s.to_owned(),
                reason: format!("key exceeds {MAX_KEY_BYTES} bytes"),
            });
        }
        if s.starts_with('/') || s.ends_with('/') {
            return Err(Error::InvalidKey {
                input: s.to_owned(),
                reason: "key must not start or end with '/'".to_owned(),
            });
        }
        for segment in s.split('/') {
            Self::validate_segment(s, segment)?;
        }
        Ok(Self(s.to_owned()))
    }

    /// Validate one `/`-separated segment of a key being parsed from
    /// `whole`.
    fn validate_segment(whole: &str, segment: &str) -> Result<(), Error> {
        if segment.is_empty() {
            return Err(Error::InvalidKey {
                input: whole.to_owned(),
                reason: "key must not contain an empty segment".to_owned(),
            });
        }
        let mut chars = segment.chars();
        let starts_ok = chars.next().is_some_and(|c| c.is_ascii_alphanumeric());
        if !starts_ok {
            return Err(Error::InvalidKey {
                input: whole.to_owned(),
                reason: format!("segment {segment:?} must start with a letter or digit"),
            });
        }
        if !chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-') {
            return Err(Error::InvalidKey {
                input: whole.to_owned(),
                reason: format!("segment {segment:?} contains an invalid character"),
            });
        }
        Ok(())
    }

    /// The key as a plain string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `true` if `self` equals `prefix`, or starts with `prefix` followed
    /// by a `/`. This is a path-segment prefix, not a string prefix:
    /// `database` is a path prefix of `database/password`, but `data` is
    /// not.
    #[must_use]
    pub fn has_path_prefix(&self, prefix: &Self) -> bool {
        self.0
            .strip_prefix(prefix.0.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    }

    /// Derive the environment variable name for this key per 3.1.4,
    /// without considering an entry's explicit `env` override: replace
    /// every `/`, `-` and `.` with `_`, uppercase ASCII letters, prepend
    /// `_` if the result would start with a digit, then prepend `prefix`
    /// verbatim.
    #[must_use]
    pub fn env_name(&self, prefix: &str) -> String {
        let mut derived: String = self
            .0
            .chars()
            .map(|c| match c {
                '/' | '-' | '.' => '_',
                other => other.to_ascii_uppercase(),
            })
            .collect();
        if derived.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            derived.insert(0, '_');
        }
        if prefix.is_empty() {
            derived
        } else {
            format!("{prefix}{derived}")
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::parse(s)
    }
}

impl TryFrom<String> for Key {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Error> {
        Self::parse(&s)
    }
}

impl From<Key> for String {
    fn from(key: Key) -> Self {
        key.0
    }
}

/// How an entry's value is represented as a JSON string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    /// The value is stored verbatim as a JSON string: valid UTF-8 without
    /// NUL bytes.
    Utf8,
    /// The value is stored as standard, padded base64 (RFC 4648 section
    /// 4).
    Base64,
}

impl Encoding {
    /// This encoding's lowercase name, matching its serialized form
    /// (`#[serde(rename_all = "lowercase")]` above) and the CLI's
    /// `encoding` field in `docs/cli.md` appendix 5.1.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf8",
            Self::Base64 => "base64",
        }
    }
}

/// The kind of store, distinguished by the age envelope header (3.1.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreKind {
    /// One or more age recipients can open the store.
    Recipients,
    /// A single passphrase (scrypt) opens the store.
    Passphrase,
}

impl StoreKind {
    /// This kind's lowercase name, matching its serialized form
    /// (`#[serde(rename_all = "lowercase")]` above) and the CLI's
    /// `kind` field in `docs/cli.md` appendix 5.1.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recipients => "recipients",
            Self::Passphrase => "passphrase",
        }
    }
}

/// Secret bytes: an entry's value.
///
/// `Value` zeroizes its contents on drop. Its [`fmt::Debug`]
/// implementation never prints the contents; it always prints
/// `Value(<redacted>)`.
///
/// `Clone` is implemented by hand: `secrecy` only implements its
/// `CloneableSecret` marker for primitive integers, not `Vec<u8>`, so
/// `SecretBox<Vec<u8>>` cannot derive `Clone`. Cloning re-wraps a copy of
/// the exposed bytes in a fresh `SecretBox`.
pub struct Value(SecretBox<Vec<u8>>);

impl Clone for Value {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.expose().to_vec())))
    }
}

impl Value {
    /// Wrap `bytes` as a secret value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooLarge`] if `bytes` exceeds
    /// [`MAX_VALUE_BYTES`].
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() > MAX_VALUE_BYTES {
            return Err(Error::TooLarge {
                bytes: bytes.len(),
                limit: MAX_VALUE_BYTES,
            });
        }
        Ok(Self(SecretBox::new(Box::new(bytes))))
    }

    /// Borrow the secret bytes. Callers must not log, print, or otherwise
    /// persist what this returns outside of the store format itself.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        self.0.expose_secret()
    }

    /// The encoding this value would be stored with: [`Encoding::Utf8`]
    /// if the bytes are valid UTF-8 without a NUL byte, [`Encoding::Base64`]
    /// otherwise.
    #[must_use]
    pub fn detect_encoding(&self) -> Encoding {
        match std::str::from_utf8(self.expose()) {
            Ok(text) if !text.contains('\0') => Encoding::Utf8,
            _ => Encoding::Base64,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Value(<redacted>)")
    }
}

/// Compares the exposed bytes. Implemented unconditionally (not only for
/// tests) because it is needed by [`Store`]'s own `PartialEq` and is
/// harmless: it does not print or log anything.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.expose() == other.expose()
    }
}

impl Eq for Value {}

/// The on-disk representation of an [`Entry`], used only to drive
/// `serde_json`. Kept private: callers use [`Entry`], never this type.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryRepr {
    value: String,
    encoding: Encoding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

impl TryFrom<EntryRepr> for Entry {
    type Error = Error;

    fn try_from(repr: EntryRepr) -> Result<Self, Error> {
        let bytes = match repr.encoding {
            Encoding::Utf8 => {
                if repr.value.contains('\0') {
                    return Err(Error::InvalidStore {
                        reason: "utf8 entry value contains a NUL byte".to_owned(),
                    });
                }
                repr.value.into_bytes()
            }
            Encoding::Base64 => {
                STANDARD
                    .decode(repr.value.as_bytes())
                    .map_err(|err| Error::InvalidStore {
                        reason: format!("invalid base64 entry value: {err}"),
                    })?
            }
        };
        Ok(Self {
            value: Value::from_bytes(bytes)?,
            encoding: repr.encoding,
            env: repr.env,
            description: repr.description,
            created_at: repr.created_at,
            updated_at: repr.updated_at,
        })
    }
}

/// Convert an [`Entry`] to its on-disk representation.
///
/// This is fallible even though `Entry`'s fields are already checked by
/// [`Store::validate`] in the normal write path: it exists as a second
/// line of defense against an `Entry` built by hand (every field is
/// `pub`) with an `encoding` that does not match its bytes.
fn entry_repr(entry: &Entry) -> Result<EntryRepr, Error> {
    let value = match entry.encoding {
        Encoding::Utf8 => {
            let text =
                std::str::from_utf8(entry.value.expose()).map_err(|_err| Error::InvalidStore {
                    reason: "utf8 entry value is not valid UTF-8".to_owned(),
                })?;
            if text.contains('\0') {
                return Err(Error::InvalidStore {
                    reason: "utf8 entry value contains a NUL byte".to_owned(),
                });
            }
            text.to_owned()
        }
        Encoding::Base64 => STANDARD.encode(entry.value.expose()),
    };
    Ok(EntryRepr {
        value,
        encoding: entry.encoding,
        env: entry.env.clone(),
        description: entry.description.clone(),
        created_at: entry.created_at,
        updated_at: entry.updated_at,
    })
}

/// One secret and its metadata, addressed by a [`Key`] inside a
/// [`Store`].
///
/// `Entry` serializes as the object described in `docs/format.md`'s
/// "Payload" section: `value` is the raw UTF-8 text or standard base64
/// depending on `encoding`, and unknown fields are rejected. `Debug`
/// redacts `value` through [`Value`]'s own `Debug` implementation.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The secret bytes.
    pub value: Value,
    /// How `value` is represented on disk.
    pub encoding: Encoding,
    /// An explicit environment variable name override (3.1.4).
    pub env: Option<String>,
    /// Free-text description, at most 1024 bytes.
    pub description: Option<String>,
    /// When this entry was first created. Never changes after that.
    pub created_at: OffsetDateTime,
    /// When this entry was last changed.
    pub updated_at: OffsetDateTime,
}

/// Compares every field, including the value (see [`Value`]'s
/// `PartialEq`). Implemented unconditionally because it is needed by
/// tests and is harmless.
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.encoding == other.encoding
            && self.env == other.env
            && self.description == other.description
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
    }
}

impl Eq for Entry {}

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        entry_repr(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        EntryRepr::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

/// A store's decrypted payload: schema, kind, recipients, and entries.
///
/// `Store` serializes exactly as described in `docs/format.md`: a JSON
/// object with sorted keys, pretty-printed, with a trailing newline (see
/// [`Store::to_json`]). `Debug` redacts every entry's value through
/// [`Entry`]'s own `Debug` implementation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    /// The schema version. Must be [`SCHEMA_VERSION`] for a store this
    /// build can use.
    pub schema: u32,
    /// Whether the store is opened with age recipients or a passphrase.
    pub kind: StoreKind,
    /// When the store was first created. Never changes after that.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// When the store was last changed.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// The recipient strings the store is sealed to. Empty for a
    /// passphrase store.
    pub recipients: Vec<String>,
    /// The store's entries, keyed by path.
    pub entries: BTreeMap<Key, Entry>,
}

/// Compares every field, including every entry's value. Implemented
/// unconditionally because it is needed by tests and is harmless.
impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        self.schema == other.schema
            && self.kind == other.kind
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
            && self.recipients == other.recipients
            && self.entries == other.entries
    }
}

impl Eq for Store {}

impl Store {
    /// Build a new, empty store of the given kind, sealed to
    /// `recipients` (empty for a passphrase store). `now` is truncated
    /// to whole seconds and used for both `created_at` and `updated_at`.
    #[must_use]
    pub fn new(kind: StoreKind, recipients: Vec<String>, now: OffsetDateTime) -> Self {
        let now = truncate_to_seconds(now);
        Self {
            schema: SCHEMA_VERSION,
            kind,
            created_at: now,
            updated_at: now,
            recipients,
            entries: BTreeMap::new(),
        }
    }

    /// Check every rule in `docs/format.md`'s "Payload" section: the
    /// schema version, `kind`/`recipients` consistency, that recipients
    /// are sorted and unique, every entry's `env` override and
    /// description, every entry's encoding/value consistency, and the
    /// total serialized size.
    ///
    /// # Errors
    ///
    /// Returns [`Error::SchemaTooNew`], [`Error::InvalidStore`], or
    /// [`Error::TooLarge`] for the first rule that is violated.
    pub fn validate(&self) -> Result<(), Error> {
        self.validate_schema()?;
        self.validate_kind()?;
        self.validate_recipients()?;
        for (key, entry) in &self.entries {
            Self::validate_entry(key, entry)?;
        }
        self.validate_size()
    }

    fn validate_schema(&self) -> Result<(), Error> {
        if self.schema > SCHEMA_VERSION {
            return Err(Error::SchemaTooNew {
                found: self.schema,
                supported: SCHEMA_VERSION,
            });
        }
        if self.schema != SCHEMA_VERSION {
            return Err(Error::InvalidStore {
                reason: format!("unsupported schema {}", self.schema),
            });
        }
        Ok(())
    }

    fn validate_kind(&self) -> Result<(), Error> {
        match self.kind {
            StoreKind::Recipients if self.recipients.is_empty() => Err(Error::InvalidStore {
                reason: "a recipients store must have at least one recipient".to_owned(),
            }),
            StoreKind::Passphrase if !self.recipients.is_empty() => Err(Error::InvalidStore {
                reason: "a passphrase store must have no recipients".to_owned(),
            }),
            StoreKind::Recipients | StoreKind::Passphrase => Ok(()),
        }
    }

    fn validate_recipients(&self) -> Result<(), Error> {
        if self.recipients.windows(2).all(|pair| pair[0] < pair[1]) {
            Ok(())
        } else {
            Err(Error::InvalidStore {
                reason: "recipients must be sorted and unique".to_owned(),
            })
        }
    }

    fn validate_entry(key: &Key, entry: &Entry) -> Result<(), Error> {
        if let Some(env) = &entry.env
            && !is_valid_env_name(env)
        {
            return Err(Error::InvalidStore {
                reason: format!("entry {key} has an invalid env override {env:?}"),
            });
        }
        if let Some(description) = &entry.description
            && description.len() > MAX_DESCRIPTION_BYTES
        {
            return Err(Error::InvalidStore {
                reason: format!("entry {key} description exceeds {MAX_DESCRIPTION_BYTES} bytes"),
            });
        }
        let bytes = entry.value.expose();
        if bytes.len() > MAX_VALUE_BYTES {
            return Err(Error::TooLarge {
                bytes: bytes.len(),
                limit: MAX_VALUE_BYTES,
            });
        }
        if matches!(entry.encoding, Encoding::Utf8) {
            let text = std::str::from_utf8(bytes).map_err(|_err| Error::InvalidStore {
                reason: format!("entry {key} is not valid UTF-8"),
            })?;
            if text.contains('\0') {
                return Err(Error::InvalidStore {
                    reason: format!("entry {key} contains a NUL byte"),
                });
            }
        }
        Ok(())
    }

    fn validate_size(&self) -> Result<(), Error> {
        let bytes = serde_json::to_vec(self).map_err(|err| Error::InvalidStore {
            reason: err.to_string(),
        })?;
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(Error::TooLarge {
                bytes: bytes.len(),
                limit: MAX_PAYLOAD_BYTES,
            });
        }
        Ok(())
    }

    /// Validate, then serialize to pretty-printed JSON with sorted keys
    /// and a trailing newline, per `docs/format.md`.
    ///
    /// # Errors
    ///
    /// Returns whatever [`Store::validate`] returns, or
    /// [`Error::TooLarge`] if the pretty-printed form (which is larger
    /// than the compact form `validate` checks) exceeds
    /// [`MAX_PAYLOAD_BYTES`].
    pub fn to_json(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|err| Error::InvalidStore {
            reason: err.to_string(),
        })?;
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(Error::TooLarge {
                bytes: bytes.len(),
                limit: MAX_PAYLOAD_BYTES,
            });
        }
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Parse and validate a store's JSON payload.
    ///
    /// Checks the raw byte length against [`MAX_PAYLOAD_BYTES`] first,
    /// then parses, then checks the schema version, then runs
    /// [`Store::validate`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooLarge`] if `bytes` exceeds
    /// [`MAX_PAYLOAD_BYTES`], [`Error::InvalidStore`] if `bytes` is not a
    /// well-formed schema 1 payload, [`Error::SchemaTooNew`] if the
    /// payload declares a newer schema, or whatever else
    /// [`Store::validate`] returns.
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(Error::TooLarge {
                bytes: bytes.len(),
                limit: MAX_PAYLOAD_BYTES,
            });
        }
        let store: Self = serde_json::from_slice(bytes).map_err(|err| Error::InvalidStore {
            reason: err.to_string(),
        })?;
        if store.schema > SCHEMA_VERSION {
            return Err(Error::SchemaTooNew {
                found: store.schema,
                supported: SCHEMA_VERSION,
            });
        }
        store.validate()?;
        Ok(store)
    }

    /// Insert or update `key`'s entry, returning `true` if it was
    /// created (it did not exist before) or `false` if an existing entry
    /// was updated.
    ///
    /// On update, `created_at` is kept, and `env` and `description` are
    /// kept unless `Some` is passed. `now` (truncated to whole seconds)
    /// becomes the entry's `updated_at` and, if the entry changed, the
    /// store's `updated_at`.
    pub fn set(
        &mut self,
        key: Key,
        value: Value,
        env: Option<String>,
        description: Option<String>,
        now: OffsetDateTime,
    ) -> bool {
        let now = truncate_to_seconds(now);
        let encoding = value.detect_encoding();
        let created = match self.entries.entry(key) {
            MapEntry::Occupied(mut occupied) => {
                let entry = occupied.get_mut();
                entry.value = value;
                entry.encoding = encoding;
                if env.is_some() {
                    entry.env = env;
                }
                if description.is_some() {
                    entry.description = description;
                }
                entry.updated_at = now;
                false
            }
            MapEntry::Vacant(vacant) => {
                vacant.insert(Entry {
                    value,
                    encoding,
                    env,
                    description,
                    created_at: now,
                    updated_at: now,
                });
                true
            }
        };
        self.updated_at = now;
        created
    }

    /// Remove `key`'s entry, if it exists, bumping the store's
    /// `updated_at` when it did.
    pub fn remove(&mut self, key: &Key, now: OffsetDateTime) -> Option<Entry> {
        let removed = self.entries.remove(key);
        if removed.is_some() {
            self.updated_at = truncate_to_seconds(now);
        }
        removed
    }

    /// Rename `from` to `to`, keeping the entry's metadata (including
    /// `created_at`) and bumping `updated_at`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::KeyNotFound`] if `from` does not exist, or
    /// [`Error::KeyExists`] if `to` already exists and `force` is
    /// `false`.
    pub fn rename(
        &mut self,
        from: &Key,
        to: Key,
        force: bool,
        now: OffsetDateTime,
    ) -> Result<(), Error> {
        if !self.entries.contains_key(from) {
            return Err(Error::KeyNotFound {
                key: from.to_string(),
            });
        }
        if !force && self.entries.contains_key(&to) {
            return Err(Error::KeyExists {
                key: to.to_string(),
            });
        }
        let now = truncate_to_seconds(now);
        if let Some(mut entry) = self.entries.remove(from) {
            entry.updated_at = now;
            self.entries.insert(to, entry);
            self.updated_at = now;
        }
        Ok(())
    }

    /// Build the environment variable mapping for `run` and `env` (3.1.4).
    ///
    /// `Base64`-encoded entries are excluded. When `only` is non-empty,
    /// only entries whose key has one of `only` as a path prefix (see
    /// [`Key::has_path_prefix`]) are included. `prefix` is prepended to
    /// every resolved name, including an entry's explicit `env`
    /// override.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidStore`] if `prefix` is non-empty and does
    /// not match `^[A-Za-z_][A-Za-z0-9_]*$`, or [`Error::EnvConflict`] if
    /// two entries resolve to the same name.
    pub fn env_map(&self, prefix: &str, only: &[Key]) -> Result<BTreeMap<String, &Entry>, Error> {
        if !prefix.is_empty() && !is_valid_env_name(prefix) {
            return Err(Error::InvalidStore {
                reason: format!("invalid env prefix {prefix:?}"),
            });
        }
        let mut sources: BTreeMap<String, Key> = BTreeMap::new();
        let mut result: BTreeMap<String, &Entry> = BTreeMap::new();
        for (key, entry) in &self.entries {
            if matches!(entry.encoding, Encoding::Base64) {
                continue;
            }
            if !only.is_empty() && !only.iter().any(|p| key.has_path_prefix(p)) {
                continue;
            }
            let name = entry.env.as_ref().map_or_else(
                || key.env_name(prefix),
                |explicit| format!("{prefix}{explicit}"),
            );
            if let Some(first_key) = sources.get(&name) {
                return Err(Error::EnvConflict {
                    name,
                    a: first_key.to_string(),
                    b: key.to_string(),
                });
            }
            sources.insert(name.clone(), key.clone());
            result.insert(name, entry);
        }
        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{Encoding, Entry, Key, Store, StoreKind, Value};
    use time::macros::datetime;

    fn now() -> time::OffsetDateTime {
        datetime!(2026-09-12 09:41:00 UTC)
    }

    fn later() -> time::OffsetDateTime {
        datetime!(2026-09-12 10:02:13 UTC)
    }

    #[test]
    fn key_accepts_valid_forms() {
        for s in ["a", "a/b", "a.b-c_d/1"] {
            assert!(Key::parse(s).is_ok(), "expected {s:?} to be valid");
        }
    }

    #[test]
    fn key_rejects_invalid_forms() {
        let long_key = "a".repeat(257);
        let cases: &[&str] = &[
            "", "/a", "a/", "a//b", "a/../b", ".hidden", "-x", &long_key, "a b", "café",
        ];
        for s in cases {
            assert!(Key::parse(s).is_err(), "expected {s:?} to be invalid");
        }
    }

    #[test]
    fn has_path_prefix() {
        let database = Key::parse("database").unwrap();
        let database_password = Key::parse("database/password").unwrap();
        let data = Key::parse("data").unwrap();
        assert!(database_password.has_path_prefix(&database));
        assert!(!database_password.has_path_prefix(&data));
        assert!(database.has_path_prefix(&database));
    }

    #[test]
    fn env_name_mapping() {
        assert_eq!(
            Key::parse("database/password").unwrap().env_name(""),
            "DATABASE_PASSWORD"
        );
        assert_eq!(Key::parse("1abc").unwrap().env_name(""), "_1ABC");
        assert_eq!(Key::parse("a.b-c").unwrap().env_name(""), "A_B_C");
        assert_eq!(
            Key::parse("database/password").unwrap().env_name("APP_"),
            "APP_DATABASE_PASSWORD"
        );
    }

    #[test]
    fn deny_unknown_fields() {
        let json = br#"{
  "schema": 1,
  "kind": "passphrase",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T09:41:00Z",
  "recipients": [],
  "entries": {},
  "extra": true
}"#;
        let err = Store::from_json(json).unwrap_err();
        assert!(matches!(err, crate::error::Error::InvalidStore { .. }));
    }

    #[test]
    fn schema_too_new() {
        let json = br#"{
  "schema": 2,
  "kind": "passphrase",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T09:41:00Z",
  "recipients": [],
  "entries": {}
}"#;
        let err = Store::from_json(json).unwrap_err();
        assert!(matches!(
            err,
            crate::error::Error::SchemaTooNew {
                found: 2,
                supported: 1
            }
        ));
    }

    #[test]
    fn kind_recipients_consistency() {
        let with_recipient_passphrase = br#"{
  "schema": 1,
  "kind": "passphrase",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T09:41:00Z",
  "recipients": ["age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"],
  "entries": {}
}"#;
        assert!(Store::from_json(with_recipient_passphrase).is_err());

        let empty_recipients = br#"{
  "schema": 1,
  "kind": "recipients",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T09:41:00Z",
  "recipients": [],
  "entries": {}
}"#;
        assert!(Store::from_json(empty_recipients).is_err());
    }

    #[test]
    fn env_map_conflict_and_explicit_override() {
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now());
        store.set(
            Key::parse("a-b").unwrap(),
            Value::from_bytes(b"1".to_vec()).unwrap(),
            None,
            None,
            now(),
        );
        store.set(
            Key::parse("a_b").unwrap(),
            Value::from_bytes(b"2".to_vec()).unwrap(),
            None,
            None,
            now(),
        );

        let conflict = store.env_map("", &[]).unwrap_err();
        assert!(matches!(conflict, crate::error::Error::EnvConflict { .. }));

        store.set(
            Key::parse("a_b").unwrap(),
            Value::from_bytes(b"2".to_vec()).unwrap(),
            Some("A_B_TWO".to_owned()),
            None,
            now(),
        );
        let map = store.env_map("", &[]).unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("A_B"));
        assert!(map.contains_key("A_B_TWO"));
    }

    #[test]
    fn env_map_excludes_base64() {
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now());
        store.set(
            Key::parse("binary").unwrap(),
            Value::from_bytes(vec![0xff, 0x00, 0xfe]).unwrap(),
            None,
            None,
            now(),
        );
        let map = store.env_map("", &[]).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn debug_redacts_value() {
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now());
        store.set(
            Key::parse("database/password").unwrap(),
            Value::from_bytes(b"hunter2".to_vec()).unwrap(),
            None,
            None,
            now(),
        );
        let rendered = format!("{store:?}");
        assert!(!rendered.contains("hunter2"));
        assert!(rendered.contains("<redacted>"));
    }

    #[test]
    fn set_reports_creation_and_update() {
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now());
        let key = Key::parse("k").unwrap();
        let created = store.set(
            key.clone(),
            Value::from_bytes(b"a".to_vec()).unwrap(),
            None,
            None,
            now(),
        );
        assert!(created);
        let updated = store.set(
            key.clone(),
            Value::from_bytes(b"b".to_vec()).unwrap(),
            None,
            None,
            later(),
        );
        assert!(!updated);
        let entry = &store.entries[&key];
        assert_eq!(entry.created_at, super::truncate_to_seconds(now()));
        assert_eq!(entry.updated_at, super::truncate_to_seconds(later()));
    }

    #[test]
    fn entry_kept_fields_on_update() {
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now());
        let key = Key::parse("k").unwrap();
        store.set(
            key.clone(),
            Value::from_bytes(b"a".to_vec()).unwrap(),
            Some("K_ENV".to_owned()),
            Some("desc".to_owned()),
            now(),
        );
        store.set(
            key.clone(),
            Value::from_bytes(b"b".to_vec()).unwrap(),
            None,
            None,
            later(),
        );
        let entry = &store.entries[&key];
        assert_eq!(entry.env.as_deref(), Some("K_ENV"));
        assert_eq!(entry.description.as_deref(), Some("desc"));
    }

    #[test]
    fn rename_key_exists_and_not_found() {
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now());
        let a = Key::parse("a").unwrap();
        let b = Key::parse("b").unwrap();
        store.set(
            a.clone(),
            Value::from_bytes(b"1".to_vec()).unwrap(),
            None,
            None,
            now(),
        );
        assert!(store.rename(&b, a.clone(), false, now()).is_err());
        store.set(
            b.clone(),
            Value::from_bytes(b"2".to_vec()).unwrap(),
            None,
            None,
            now(),
        );
        assert!(store.rename(&a, b.clone(), false, now()).is_err());
        assert!(store.rename(&a, b.clone(), true, now()).is_ok());
    }

    #[test]
    fn entry_value_round_trips_utf8_and_base64() {
        let utf8_entry = Entry {
            value: Value::from_bytes(b"s3cr3t".to_vec()).unwrap(),
            encoding: Encoding::Utf8,
            env: None,
            description: None,
            created_at: super::truncate_to_seconds(now()),
            updated_at: super::truncate_to_seconds(now()),
        };
        let json = serde_json::to_string(&utf8_entry).unwrap();
        assert!(json.contains("\"s3cr3t\""));
        let parsed: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value.expose(), b"s3cr3t");

        let binary_entry = Entry {
            value: Value::from_bytes(vec![0xff, 0xec, 0x20]).unwrap(),
            encoding: Encoding::Base64,
            env: None,
            description: None,
            created_at: super::truncate_to_seconds(now()),
            updated_at: super::truncate_to_seconds(now()),
        };
        let json = serde_json::to_string(&binary_entry).unwrap();
        let parsed: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value.expose(), &[0xff, 0xec, 0x20]);
    }

    #[test]
    fn entry_encoding_mismatch_is_invalid_store() {
        let bad = br#"{
  "value": "not base64!!",
  "encoding": "base64",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T09:41:00Z"
}"#;
        let err = serde_json::from_slice::<Entry>(bad).unwrap_err();
        assert!(err.to_string().contains("base64"));
    }
}
