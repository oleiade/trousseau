//! `trousseau get` (3.5.5), including `--clip` (3.5.16, step 3.9).

use anyhow::Context as _;
use base64::Engine as _;
use serde::Serialize;

use trousseau::schema::{Encoding, Key};
use trousseau::store::LockMode;

use crate::cli::GetArgs;
use crate::context::Context;
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
            super::write_private_file(out_path, bytes, args.force)?;
        }
        return output::json(&GetJson {
            key: key.as_str(),
            value: stored_representation(entry.encoding, bytes)?,
            encoding: entry.encoding.as_str(),
            env: entry.env.as_deref(),
            description: entry.description.as_deref(),
            created_at: super::format_rfc3339(entry.created_at)?,
            updated_at: super::format_rfc3339(entry.updated_at)?,
        });
    }

    if let Some(out_path) = &args.out {
        return super::write_private_file(out_path, bytes, args.force);
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
