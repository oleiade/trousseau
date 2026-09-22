//! `trousseau ls` (3.5.6).

use serde::Serialize;

use trousseau::schema::{Entry, Key};
use trousseau::store::LockMode;

use crate::cli::LsArgs;
use crate::context::Context;
use crate::output::{self, OutputMode};

/// The `--long` table's column headers (3.5.6). Never a value column.
const LONG_HEADERS: [&str; 5] = ["KEY", "ENC", "ENV", "UPDATED", "DESCRIPTION"];

/// Run `ls`.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::InvalidKey`] if `PREFIX` does not
/// satisfy the key grammar, or whatever [`Context::unlock`] returns.
pub fn run(ctx: &Context, args: &LsArgs) -> anyhow::Result<()> {
    let prefix = args.prefix.as_deref().map(Key::parse).transpose()?;

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Shared)?;
    let store = ctx.unlock(path)?;

    let matches: Vec<(&Key, &Entry)> = store
        .entries
        .iter()
        .filter(|(key, _)| prefix.as_ref().is_none_or(|p| key.has_path_prefix(p)))
        .collect();

    match ctx.output {
        OutputMode::Human if args.long => print_long(&matches),
        OutputMode::Human => {
            for (key, _) in &matches {
                output::line(key.as_str());
            }
            Ok(())
        }
        OutputMode::Json => print_json(&matches),
    }
}

/// The `--long` human table: `KEY`, `ENC`, `ENV`, `UPDATED`,
/// `DESCRIPTION`, never a value (3.5.6). Prints nothing, not even the
/// header, when `matches` is empty ("An empty result is exit 0 with
/// empty output").
fn print_long(matches: &[(&Key, &Entry)]) -> anyhow::Result<()> {
    if matches.is_empty() {
        return Ok(());
    }
    let mut rows = Vec::with_capacity(matches.len());
    for (key, entry) in matches {
        rows.push(vec![
            key.to_string(),
            entry.encoding.as_str().to_owned(),
            entry.env.clone().unwrap_or_default(),
            super::format_rfc3339(entry.updated_at)?,
            entry.description.clone().unwrap_or_default(),
        ]);
    }
    output::table(&LONG_HEADERS, &rows);
    Ok(())
}

/// The `ls` output shape for `--json` (appendix 5.1): one entry per
/// element, the same shape as `get --json` minus `value`.
#[derive(Serialize)]
struct LsEntryJson<'a> {
    key: &'a str,
    encoding: &'static str,
    env: Option<&'a str>,
    description: Option<&'a str>,
    created_at: String,
    updated_at: String,
}

fn print_json(matches: &[(&Key, &Entry)]) -> anyhow::Result<()> {
    let mut items = Vec::with_capacity(matches.len());
    for (key, entry) in matches {
        items.push(LsEntryJson {
            key: key.as_str(),
            encoding: entry.encoding.as_str(),
            env: entry.env.as_deref(),
            description: entry.description.as_deref(),
            created_at: super::format_rfc3339(entry.created_at)?,
            updated_at: super::format_rfc3339(entry.updated_at)?,
        });
    }
    output::json(&items)
}
