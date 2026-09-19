//! Trousseau is a portable, encrypted keyring for storing and sharing
//! secrets from the command line.
//!
//! This crate implements everything that touches the store format, the
//! age based envelope cryptography, the filesystem representation of a
//! store, and the legacy v0.4 reader. It performs no terminal I/O, holds
//! no identities or prompts, spawns no process except `gpg` for the
//! legacy reader, and has no knowledge of environment variables. It
//! never prints. That behavior lives in the `trousseau-cli` binary
//! crate.

/// Typed errors returned by every fallible operation in this crate.
pub mod error;

/// The store format: schema 1 payload, keys, encodings, and the
/// environment variable mapping. See `docs/format.md`.
pub mod schema;

/// The age envelope: sealing and opening store bytes. See
/// `docs/format.md` section on the envelope.
pub mod envelope;

/// Recipients and identities: parsing, normalizing, and loading the age
/// recipients and identity sources described in section 3.3 of
/// `docs/IMPLEMENTATION_PLAN.md`.
pub mod identity;
