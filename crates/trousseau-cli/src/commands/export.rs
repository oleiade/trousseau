//! `trousseau export` (3.5.11).

use std::path::Path;

use anyhow::Context as _;

use trousseau::schema::{Encoding, Store};
use trousseau::store::LockMode;

use crate::cli::{ExportArgs, PayloadFormat};
use crate::context::Context;
use crate::exit::CliError;
use crate::output;

/// Run `export`.
///
/// # Errors
///
/// Returns [`CliError::Usage`] if global `--json` is combined with a
/// `--format` other than `json`, [`CliError::OutputExists`] if `--out`
/// names an existing file without `--force`, or whatever
/// [`Context::unlock`] or writing the output returns.
pub fn run(ctx: &Context, args: &ExportArgs) -> anyhow::Result<()> {
    // `--json` is 3.5.1's global "wrap the result as one JSON document"
    // flag; `export`'s own `--format json` already *is* that document
    // (appendix 5.1: "the payload document (format json), or rejected
    // with exit 2 for other formats"), so the two only make sense
    // together.
    if ctx.output == crate::output::OutputMode::Json && args.format != PayloadFormat::Json {
        return Err(CliError::Usage(format!(
            "--json can only be combined with --format json, not --format {}",
            format_label(args.format)
        ))
        .into());
    }

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Shared)?;
    let store = ctx.unlock(path)?;

    let bytes = match args.format {
        PayloadFormat::Json => store.to_json()?,
        PayloadFormat::Dotenv => build_dotenv(&store).into_bytes(),
        PayloadFormat::Toml => crate::document::to_toml(&store, "# trousseau export").into_bytes(),
    };

    if let Some(out_path) = &args.out {
        write_out_file(out_path, &bytes, args.force)?;
        output::warn(&format!(
            "warning: {} contains plaintext secrets",
            out_path.display()
        ));
        return Ok(());
    }

    output::raw(&bytes)
}

/// The `--format` value's own spelling, for an error message (3.5.11).
const fn format_label(format: PayloadFormat) -> &'static str {
    match format {
        PayloadFormat::Json => "json",
        PayloadFormat::Dotenv => "dotenv",
        PayloadFormat::Toml => "toml",
    }
}

/// Render `store` as `NAME="value"` lines, one per `utf8` entry, in key
/// order (3.5.11). A `base64` entry is skipped, with a stderr warning
/// printed for each.
fn build_dotenv(store: &Store) -> String {
    let mut out = String::new();
    for (key, entry) in &store.entries {
        match entry.encoding {
            Encoding::Base64 => {
                output::warn(&format!("warning: skipping binary entry {key}"));
            }
            Encoding::Utf8 => {
                let name = entry.env.clone().unwrap_or_else(|| key.env_name(""));
                // A `utf8` entry's bytes are always valid UTF-8 (checked
                // at every write path); `unwrap_or_default` is a
                // defensive fallback, never expected to trigger.
                let text = std::str::from_utf8(entry.value.expose()).unwrap_or_default();
                out.push_str(&name);
                out.push_str("=\"");
                out.push_str(&escape_dotenv_value(text));
                out.push_str("\"\n");
            }
        }
    }
    out
}

/// Escape `"`, `\`, and a newline as `\"`, `\\`, `\n` (3.5.11).
fn escape_dotenv_value(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Write `bytes` to `path` with mode `0600`, refusing to overwrite an
/// existing file unless `force` (3.5.11). Mirrors `get`'s `--out`
/// handling (3.5.5).
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

/// Set `file`'s permissions to `0600` (3.5.11). A no-op on Windows,
/// which relies on the user profile's ACLs (3.2).
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
