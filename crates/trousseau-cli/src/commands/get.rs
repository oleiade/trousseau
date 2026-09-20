//! `trousseau get` (3.5.5), including `--clip` (3.5.16, step 3.9).

use std::path::Path;

use anyhow::Context as _;
use base64::Engine as _;
use serde::Serialize;

use trousseau::schema::{Encoding, Key};
use trousseau::store::LockMode;

use crate::cli::GetArgs;
use crate::context::Context;
use crate::exit::CliError;
use crate::output::{self, OutputMode};

use super::clip;

/// Run `get`.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::InvalidKey`] if `KEY` does not
/// satisfy the key grammar, [`trousseau::error::Error::KeyNotFound`] if
/// it does not exist in the store, [`CliError::OutputExists`] if
/// `--out` names an existing file without `--force`, "built without
/// clipboard support" if `--clip` is given and this build lacks the
/// `clipboard` feature (3.5.5), or whatever [`Context::unlock`],
/// [`clip::copy_to_clipboard`], or writing the output returns.
pub fn run(ctx: &Context, args: &GetArgs) -> anyhow::Result<()> {
    let key = Key::parse(&args.key)?;
    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Shared)?;

    let store = ctx.unlock(path)?;
    let entry = store
        .entries
        .get(&key)
        .ok_or_else(|| trousseau::error::Error::KeyNotFound {
            key: key.to_string(),
        })?;
    let bytes = entry.value.expose();

    if args.clip {
        clip::copy_to_clipboard(ctx, key.as_str(), entry.encoding, bytes)?;
    }

    if ctx.output == OutputMode::Json {
        if let Some(out_path) = &args.out {
            write_out_file(out_path, bytes, args.force)?;
        }
        return output::json(&GetJson {
            key: key.as_str(),
            value: stored_representation(entry.encoding, bytes)?,
            encoding: super::encoding_label(entry.encoding),
            env: entry.env.as_deref(),
            description: entry.description.as_deref(),
            created_at: super::format_rfc3339(entry.created_at)?,
            updated_at: super::format_rfc3339(entry.updated_at)?,
        });
    }

    if let Some(out_path) = &args.out {
        return write_out_file(out_path, bytes, args.force);
    }

    if args.clip {
        return Ok(());
    }

    print_raw(ctx, entry.encoding, bytes)
}

/// The default (no `--out`, no `--json`) stdout behavior (3.5.5).
fn print_raw(ctx: &Context, encoding: Encoding, bytes: &[u8]) -> anyhow::Result<()> {
    match encoding {
        Encoding::Utf8 => {
            output::raw(bytes)?;
            if ctx.is_stdout_tty {
                output::raw(b"\n")?;
            }
            Ok(())
        }
        Encoding::Base64 => {
            if ctx.is_stdout_tty {
                Err(anyhow::anyhow!(
                    "binary value; use --out or pipe the output"
                ))
            } else {
                output::raw(bytes)
            }
        }
    }
}

/// Write `bytes` to `path` with mode `0600` (3.5.5).
///
/// # Errors
///
/// Returns [`CliError::OutputExists`] if `path` already exists and
/// `force` is `false`, or an I/O error otherwise.
fn write_out_file(path: &Path, bytes: &[u8], force: bool) -> anyhow::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let file = match options.open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(CliError::OutputExists {
                path: path.to_path_buf(),
            }
            .into());
        }
        Err(err) => return Err(err).with_context(|| format!("writing {}", path.display())),
    };
    set_out_file_mode(&file, path)?;
    write_bytes(file, path, bytes)
}

/// Set `file`'s permissions to `0600` (3.5.5). A no-op on Windows, which
/// relies on the user profile's ACLs (3.2).
#[cfg(unix)]
fn set_out_file_mode(file: &std::fs::File, path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("setting permissions on {}", path.display()))
}

#[cfg(not(unix))]
fn set_out_file_mode(_file: &std::fs::File, _path: &Path) -> anyhow::Result<()> {
    Ok(())
}

/// Write `bytes` to an already-opened `file`.
fn write_bytes(mut file: std::fs::File, path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::io::Write as _;
    file.write_all(bytes)
        .with_context(|| format!("writing {}", path.display()))
}

/// The entry's value in its stored JSON representation (3.1.2, 3.5.5):
/// the UTF-8 text for a `utf8` entry, or standard base64 for a `base64`
/// entry.
fn stored_representation(encoding: Encoding, bytes: &[u8]) -> anyhow::Result<String> {
    match encoding {
        Encoding::Utf8 => {
            String::from_utf8(bytes.to_vec()).context("utf8 entry value is not valid UTF-8")
        }
        Encoding::Base64 => Ok(base64::engine::general_purpose::STANDARD.encode(bytes)),
    }
}

/// The `get` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct GetJson<'a> {
    key: &'a str,
    value: String,
    encoding: &'static str,
    env: Option<&'a str>,
    description: Option<&'a str>,
    created_at: String,
    updated_at: String,
}
