//! The error catalogue for the trousseau library.
//!
//! Every variant's message is safe to print: none of them ever contain a
//! secret value, a passphrase, or identity material. See
//! `docs/IMPLEMENTATION_PLAN.md` section 3.6 for the normative source of
//! this enum.

use std::path::PathBuf;

/// The error type returned by every fallible operation in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No store file exists at the resolved path.
    #[error("store not found at {path}")]
    StoreNotFound {
        /// The path that was searched.
        path: PathBuf,
    },

    /// The store at `path` is a legacy v0.4 store and must be migrated
    /// with `trousseau migrate` before it can be opened.
    #[error("legacy v0.4 store detected at {path}; run 'trousseau migrate'")]
    LegacyStore {
        /// The path of the legacy store.
        path: PathBuf,
    },

    /// The store's contents do not satisfy the schema 1 rules.
    #[error("invalid store: {reason}")]
    InvalidStore {
        /// A human-readable explanation of what is invalid.
        reason: String,
    },

    /// The store declares a schema version newer than this build
    /// understands.
    #[error("store schema {found} is newer than this build supports ({supported})")]
    SchemaTooNew {
        /// The schema version found in the store.
        found: u32,
        /// The highest schema version this build supports.
        supported: u32,
    },

    /// The store could not be decrypted: wrong passphrase, no matching
    /// identity, or a plugin failure.
    #[error("cannot unlock store: {reason}")]
    Unlock {
        /// A human-readable explanation, never containing secret
        /// material.
        reason: String,
    },

    /// No identity source was configured or found.
    #[error("no identity available")]
    NoIdentity,

    /// A recipient string could not be parsed.
    #[error("invalid recipient: {input}: {reason}")]
    InvalidRecipient {
        /// The offending recipient string.
        input: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A key string does not satisfy the key grammar.
    #[error("invalid key: {input}: {reason}")]
    InvalidKey {
        /// The offending key string.
        input: String,
        /// Why it was rejected.
        reason: String,
    },

    /// The requested key does not exist in the store.
    #[error("key not found: {key}")]
    KeyNotFound {
        /// The missing key.
        key: String,
    },

    /// The key already exists and the operation would overwrite it
    /// without permission.
    #[error("key already exists: {key}")]
    KeyExists {
        /// The colliding key.
        key: String,
    },

    /// Two entries resolve to the same environment variable name.
    #[error("environment name {name} is produced by both {a} and {b}")]
    EnvConflict {
        /// The conflicting environment variable name.
        name: String,
        /// The first key that produces `name`.
        a: String,
        /// The second key that produces `name`.
        b: String,
    },

    /// Another process holds the store lock past the wait timeout.
    #[error("store is locked by another process")]
    LockTimeout,

    /// A value or payload exceeds the configured size limit.
    #[error("value too large: {bytes} bytes (limit {limit})")]
    TooLarge {
        /// The size that was rejected, in bytes.
        bytes: usize,
        /// The maximum allowed size, in bytes.
        limit: usize,
    },

    /// An error occurred while reading a legacy v0.4 store.
    #[error("legacy store error: {reason}")]
    Legacy {
        /// A human-readable explanation.
        reason: String,
    },

    /// An identity file's contents could not be recognized or parsed.
    #[error("invalid identity file {path}: {reason}")]
    InvalidIdentity {
        /// The path of the offending identity file.
        path: PathBuf,
        /// Why it was rejected.
        reason: String,
    },

    /// An underlying I/O operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn key_not_found_display() {
        let err = Error::KeyNotFound {
            key: "foo".to_owned(),
        };
        assert_eq!(err.to_string(), "key not found: foo");
    }
}
