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
