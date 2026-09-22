//! `trousseau set` (3.5.4).

use std::io::Read as _;
use std::path::Path;

use anyhow::Context as _;
use secrecy::ExposeSecret as _;
use serde::Serialize;

use trousseau::schema::{Encoding, Key, Value};
use trousseau::store::LockMode;

use crate::cli::SetArgs;
use crate::context::Context;
use crate::exit::CliError;
use crate::output::{self, OutputMode};
use crate::prompt;

/// The error for `set KEY VALUE`. Never includes the value.
const VALUE_ON_COMMAND_LINE: &str = "\
set takes the value from a prompt, stdin, a file, or an environment variable, never from the command line (it would land in your shell history and the process list).
  prompt:  trousseau set KEY
  pipe:    printf %s 'value' | trousseau set KEY
  file:    trousseau set KEY --from-file PATH
The value you just typed is now in your shell history. Remove it from there if it is a real secret.";

/// Run `set`.
///
/// # Errors
///
/// Returns [`CliError::Usage`] if a value was given on the command line
/// (`set KEY VALUE`, 3.5.4: the value is never accepted there),
/// [`trousseau::error::Error::InvalidKey`] if `KEY` does not satisfy the
/// key grammar, an error if `--from-env` names a variable that is not
/// set, whatever reading `--from-file` or stdin returns, whatever
/// [`Context::unlock`], [`Context::seal_for`], or
/// [`trousseau::store::save`] returns.
pub fn run(ctx: &Context, args: &SetArgs) -> anyhow::Result<()> {
    if !args.rejected_value.is_empty() {
        return Err(CliError::Usage(VALUE_ON_COMMAND_LINE.to_owned()).into());
    }

    let key = Key::parse(&args.key)?;
    let resolved = ctx.resolve_store();
    let path = resolved.path();

    // The lock is acquired before the value is read: `set` with a
    // terminal-less, still-open stdin blocks here (rule 4 below), and a
    // concurrent reader must wait for that same lock (3.5.1, step 3.3's
    // lock test).
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    let bytes = read_value(ctx, args)?;
    let value = Value::from_bytes(bytes)?;

    let mut store = ctx.unlock(path)?;
    let now = ctx.now();
    let created = store.set(
        key.clone(),
        value,
        args.env.clone(),
        args.description.clone(),
        now,
    );
    if args.binary
        && let Some(entry) = store.entries.get_mut(&key)
    {
        entry.encoding = Encoding::Base64;
    }
    let encoding = store
        .entries
        .get(&key)
        .map(|entry| entry.encoding)
        .ok_or_else(|| anyhow::anyhow!("internal error: {key} vanished after set"))?;

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    report(ctx, &key, encoding, created)
}

/// Resolve `set`'s value source, per 3.5.4:
///
/// 1. `--from-file PATH` (or stdin, verbatim, if `PATH` is `-`).
/// 2. `--from-env NAME`.
/// 3. An interactive hidden prompt, if stdin is a terminal.
/// 4. Otherwise, all of stdin with exactly one trailing `\n` or `\r\n`
///    stripped.
fn read_value(ctx: &Context, args: &SetArgs) -> anyhow::Result<Vec<u8>> {
    if let Some(path) = &args.from_file {
        return read_from_file(path);
    }
    if let Some(name) = &args.from_env {
        return std::env::var(name)
            .map(String::into_bytes)
            .with_context(|| format!("reading environment variable {name}"));
    }
    if ctx.is_stdin_tty {
        if ctx.no_input {
            return Err(trousseau::error::Error::Unlock {
                reason: "no value available (--no-input)".to_owned(),
            }
            .into());
        }
        let secret = prompt::hidden(&format!("Value for {}: ", args.key))?;
        return Ok(secret.expose_secret().as_bytes().to_vec());
    }
    let bytes = read_stdin_to_end()?;
    Ok(strip_trailing_newline(bytes))
}

/// `--from-file PATH`: file bytes verbatim, or (`PATH` is `-`) all of
/// stdin verbatim, with no newline stripping either way (3.5.4).
fn read_from_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    if path.as_os_str() == "-" {
        return read_stdin_to_end();
    }
    std::fs::read(path).with_context(|| format!("reading {}", path.display()))
}

/// Read all of stdin to a byte buffer.
fn read_stdin_to_end() -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    std::io::stdin()
        .lock()
        .read_to_end(&mut buf)
        .context("reading stdin")?;
    Ok(buf)
}

/// Strip exactly one trailing `\r\n` or `\n` from `bytes` (3.5.4, rule 4).
fn strip_trailing_newline(mut bytes: Vec<u8>) -> Vec<u8> {
    if bytes.ends_with(b"\r\n") {
        bytes.truncate(bytes.len() - 2);
    } else if bytes.ends_with(b"\n") {
        bytes.truncate(bytes.len() - 1);
    }
    bytes
}

/// The `set` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct SetJson<'a> {
    ok: bool,
    key: &'a str,
    encoding: &'static str,
    created: bool,
}

/// Print `set`'s result: `set KEY` on stderr in human mode, or the JSON
/// shape from appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, key: &Key, encoding: Encoding, created: bool) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            output::info(ctx.quiet, &format!("set {key}"));
            Ok(())
        }
        OutputMode::Json => output::json(&SetJson {
            ok: true,
            key: key.as_str(),
            encoding: encoding.as_str(),
            created,
        }),
    }
}
