//! Exit codes and error reporting (3.5.1).
//!
//! [`code_for`] maps an [`anyhow::Error`] to the process exit code from
//! the table in `docs/cli.md`. [`report`] prints the error, in the
//! shape the current output mode calls for, through `output.rs`. Clap's
//! own usage errors never reach this module: `main.rs` lets
//! [`clap::Error::exit`] handle them directly, keeping clap's exit code
//! (2) and its own message, per 3.5.1.

use std::path::PathBuf;

use trousseau::error::Error;

use crate::output::{self, OutputMode};

/// A CLI-level error that does not originate in the `trousseau` library.
///
/// [`Error::KeyExists`] is about a store *entry*'s key, not the store
/// file itself, so `init`'s "the target store already exists" condition
/// (3.5.2) is not a library error at all: it is caught by the CLI before
/// any store is opened, and reported here as [`CliError::StoreExists`].
/// [`CliError::Usage`] covers a command-line combination clap's own
/// grammar cannot express as invalid, such as `init --no-self` with no
/// recipients (3.5.2). [`CliError::Refused`] and [`CliError::ChildFailed`]
/// are not constructed until the commands that need them land in a later
/// step (`recipients rm`'s confirmation, and `run`'s child-process
/// handling); `code_for` and `json_code_for` already handle both so
/// those steps need no changes here.
#[allow(dead_code)]
#[derive(Debug)]
pub enum CliError {
    /// The user declined a confirmation, or one was refused
    /// automatically because `--no-input` (or a non-terminal stdin) was
    /// in effect.
    Refused,
    /// A child process spawned by `run` could not be waited on, or (on
    /// Windows) reported an unrepresentable exit status. Does not cover
    /// the child's own exit code, which `run` passes through directly.
    ChildFailed,
    /// `init`'s target store already exists (3.5.2).
    StoreExists {
        /// The store path that already exists.
        path: PathBuf,
    },
    /// A usage error this crate's own validation caught, rather than
    /// clap's grammar (3.5.2: `--no-self` with no recipients).
    Usage(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused => f.write_str("refused"),
            Self::ChildFailed => f.write_str("child process failed"),
            Self::StoreExists { path } => {
                write!(f, "store already exists at {}", path.display())
            }
            Self::Usage(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for CliError {}

/// The process exit code for `err`, per the table in `docs/cli.md`.
///
/// Downcasts to [`trousseau::error::Error`] first, then to [`CliError`]; any
/// other error (including every "not implemented yet" stub in this
/// step) is exit code 1, the generic failure code.
#[must_use]
pub fn code_for(err: &anyhow::Error) -> i32 {
    if let Some(lib_err) = err.downcast_ref::<Error>() {
        return code_for_lib_error(lib_err);
    }
    if let Some(cli_err) = err.downcast_ref::<CliError>() {
        return match cli_err {
            CliError::Refused | CliError::Usage(_) => 2,
            CliError::ChildFailed => 1,
            CliError::StoreExists { .. } => 8,
        };
    }
    1
}

/// The exit code for one [`trousseau::error::Error`] variant, per the table in
/// `docs/cli.md`. Exhaustive: a new variant fails this build, not a
/// silently-wrong exit code.
const fn code_for_lib_error(err: &Error) -> i32 {
    match err {
        Error::StoreNotFound { .. } => 3,
        Error::LegacyStore { .. } => 7,
        Error::Unlock { .. } | Error::NoIdentity => 4,
        Error::KeyNotFound { .. } => 5,
        Error::LockTimeout => 6,
        Error::KeyExists { .. } | Error::EnvConflict { .. } => 8,
        Error::InvalidStore { .. }
        | Error::SchemaTooNew { .. }
        | Error::InvalidRecipient { .. }
        | Error::InvalidKey { .. }
        | Error::TooLarge { .. }
        | Error::Legacy { .. }
        | Error::InvalidIdentity { .. }
        | Error::Io(_) => 1,
    }
}

/// The JSON error `code` for `err` (appendix 5.1): a [`trousseau::error::Error`]
/// variant's name in `snake_case`, `"refused"` or `"child_failed"` for a
/// [`CliError`], or `"error"` for anything else (including a
/// "not implemented yet" stub).
#[must_use]
pub fn json_code_for(err: &anyhow::Error) -> &'static str {
    if let Some(lib_err) = err.downcast_ref::<Error>() {
        return json_code_for_lib_error(lib_err);
    }
    if let Some(cli_err) = err.downcast_ref::<CliError>() {
        return match cli_err {
            CliError::Refused => "refused",
            CliError::ChildFailed => "child_failed",
            CliError::StoreExists { .. } => "store_exists",
            CliError::Usage(_) => "usage",
        };
    }
    "error"
}

/// The `snake_case` JSON code for one [`trousseau::error::Error`] variant.
/// Exhaustive, for the same reason as [`code_for_lib_error`].
const fn json_code_for_lib_error(err: &Error) -> &'static str {
    match err {
        Error::StoreNotFound { .. } => "store_not_found",
        Error::LegacyStore { .. } => "legacy_store",
        Error::InvalidStore { .. } => "invalid_store",
        Error::SchemaTooNew { .. } => "schema_too_new",
        Error::Unlock { .. } => "unlock",
        Error::NoIdentity => "no_identity",
        Error::InvalidRecipient { .. } => "invalid_recipient",
        Error::InvalidKey { .. } => "invalid_key",
        Error::KeyNotFound { .. } => "key_not_found",
        Error::KeyExists { .. } => "key_exists",
        Error::EnvConflict { .. } => "env_conflict",
        Error::LockTimeout => "lock_timeout",
        Error::TooLarge { .. } => "too_large",
        Error::Legacy { .. } => "legacy",
        Error::InvalidIdentity { .. } => "invalid_identity",
        Error::Io(_) => "io",
    }
}

/// Report `err` through `output.rs`, in the shape `mode` calls for
/// (3.5.1): `error: <message>` on stderr in human mode, or the JSON
/// error object on stderr (stdout stays empty) in JSON mode.
pub fn report(err: &anyhow::Error, mode: OutputMode) {
    let message = err.to_string();
    match mode {
        OutputMode::Human => output::error_human(&message),
        OutputMode::Json => output::error_json(json_code_for(err), &message),
    }
}

#[cfg(test)]
mod tests {
    use super::code_for;
    use trousseau::error::Error;

    #[test]
    fn every_trousseau_error_variant_has_the_documented_exit_code() {
        let cases: Vec<(Error, i32)> = vec![
            (Error::StoreNotFound { path: "x".into() }, 3),
            (Error::LegacyStore { path: "x".into() }, 7),
            (
                Error::InvalidStore {
                    reason: "x".to_owned(),
                },
                1,
            ),
            (
                Error::SchemaTooNew {
                    found: 2,
                    supported: 1,
                },
                1,
            ),
            (
                Error::Unlock {
                    reason: "x".to_owned(),
                },
                4,
            ),
            (Error::NoIdentity, 4),
            (
                Error::InvalidRecipient {
                    input: "x".to_owned(),
                    reason: "x".to_owned(),
                },
                1,
            ),
            (
                Error::InvalidKey {
                    input: "x".to_owned(),
                    reason: "x".to_owned(),
                },
                1,
            ),
            (
                Error::KeyNotFound {
                    key: "x".to_owned(),
                },
                5,
            ),
            (
                Error::KeyExists {
                    key: "x".to_owned(),
                },
                8,
            ),
            (
                Error::EnvConflict {
                    name: "x".to_owned(),
                    a: "x".to_owned(),
                    b: "y".to_owned(),
                },
                8,
            ),
            (Error::LockTimeout, 6),
            (Error::TooLarge { bytes: 1, limit: 1 }, 1),
            (
                Error::Legacy {
                    reason: "x".to_owned(),
                },
                1,
            ),
            (
                Error::InvalidIdentity {
                    path: "x".into(),
                    reason: "x".to_owned(),
                },
                1,
            ),
            (Error::Io(std::io::Error::other("x")), 1),
        ];
        for (variant, expected) in cases {
            let wrapped: anyhow::Error = variant.into();
            assert_eq!(code_for(&wrapped), expected, "{wrapped}");
        }
    }

    #[test]
    fn non_library_error_is_generic_failure() {
        let err = anyhow::anyhow!("not implemented yet (step 3.2)");
        assert_eq!(code_for(&err), 1);
        assert_eq!(super::json_code_for(&err), "error");
    }

    #[test]
    fn cli_error_codes() {
        let refused: anyhow::Error = super::CliError::Refused.into();
        assert_eq!(code_for(&refused), 2);
        assert_eq!(super::json_code_for(&refused), "refused");

        let child_failed: anyhow::Error = super::CliError::ChildFailed.into();
        assert_eq!(code_for(&child_failed), 1);
        assert_eq!(super::json_code_for(&child_failed), "child_failed");

        let store_exists: anyhow::Error = super::CliError::StoreExists { path: "x".into() }.into();
        assert_eq!(code_for(&store_exists), 8);
        assert_eq!(super::json_code_for(&store_exists), "store_exists");

        let usage: anyhow::Error = super::CliError::Usage("x".to_owned()).into();
        assert_eq!(code_for(&usage), 2);
        assert_eq!(super::json_code_for(&usage), "usage");
    }
}
