//! `trousseau import` (3.5.11).

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::Path;

use anyhow::Context as _;
use serde::Serialize;
use time::OffsetDateTime;

use trousseau::schema::{Encoding, Entry, Key, Store, Value};
use trousseau::store::LockMode;

use crate::cli::{ImportArgs, ImportStrategy, PayloadFormat};
use crate::context::Context;
use crate::document::DocEntry;
use crate::output::{self, OutputMode};

/// Every entry parsed out of an import source, before it is applied to
/// the store. The `created_at` beside each [`DocEntry`] is `Some` only
/// for `--format json`, the one format that carries a timestamp to
/// preserve; `updated_at` is always "now" on import (3.5.11).
type Imported = BTreeMap<Key, (DocEntry, Option<OffsetDateTime>)>;

/// Run `import`.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::KeyExists`] on any collision under
/// the default `fail` strategy (nothing is written), whatever parsing
/// the chosen `--format` returns, or whatever [`Context::unlock`],
/// [`Context::seal_for`], or [`trousseau::store::save`] returns.
pub fn run(ctx: &Context, args: &ImportArgs) -> anyhow::Result<()> {
    let bytes = read_input(args.path.as_deref())?;
    let imported = parse_import(args.format, &bytes)?;

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Exclusive)?;
    let mut store = ctx.unlock(path)?;
    let now = trousseau::schema::truncate_to_seconds(ctx.now());

    // `fail` (the default) is all-or-nothing: check every imported key
    // against the store before mutating anything, so a collision leaves
    // the store untouched (Step 3.5's "no partial writes on `fail`").
    if args.strategy == ImportStrategy::Fail {
        for key in imported.keys() {
            if store.entries.contains_key(key) {
                return Err(trousseau::error::Error::KeyExists {
                    key: key.to_string(),
                }
                .into());
            }
        }
    }

    let (added, updated, skipped) = apply_imported(&mut store, imported, args.strategy, now);
    if added > 0 || updated > 0 {
        store.updated_at = now;
    }

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    report(ctx, added, updated, skipped)
}

/// Apply every imported entry to `store` per `strategy`, returning
/// `(added, updated, skipped)`.
fn apply_imported(
    store: &mut Store,
    imported: Imported,
    strategy: ImportStrategy,
    now: OffsetDateTime,
) -> (u64, u64, u64) {
    let mut added = 0u64;
    let mut updated = 0u64;
    let mut skipped = 0u64;
    for (key, (doc, created_at)) in imported {
        let existed = store.entries.contains_key(&key);
        if existed && strategy == ImportStrategy::Keep {
            skipped += 1;
            continue;
        }
        store.entries.insert(
            key,
            Entry {
                value: doc.value,
                encoding: doc.encoding,
                env: doc.env,
                description: doc.description,
                created_at: created_at.unwrap_or(now),
                updated_at: now,
            },
        );
        if existed {
            updated += 1;
        } else {
            added += 1;
        }
    }
    (added, updated, skipped)
}

/// Read the import source: `path`, or all of stdin when `path` is
/// `None` (3.5.11).
fn read_input(path: Option<&Path>) -> anyhow::Result<Vec<u8>> {
    if let Some(path) = path {
        return std::fs::read(path).with_context(|| format!("reading {}", path.display()));
    }
    let mut buf = Vec::new();
    std::io::stdin()
        .lock()
        .read_to_end(&mut buf)
        .context("reading stdin")?;
    Ok(buf)
}

/// Parse `bytes` into one entry per key, per `format`.
fn parse_import(format: PayloadFormat, bytes: &[u8]) -> anyhow::Result<Imported> {
    match format {
        PayloadFormat::Json => parse_json(bytes),
        PayloadFormat::Dotenv => parse_dotenv(bytes),
        PayloadFormat::Toml => parse_toml(bytes),
    }
}

/// `--format json`: the full payload document (3.1.2, 3.5.11). `kind`,
/// `recipients`, and the store-level timestamps are parsed (so the
/// document is validated as a whole) and then ignored; only `entries`
/// is used, keeping each entry's `created_at`.
fn parse_json(bytes: &[u8]) -> anyhow::Result<Imported> {
    let doc = Store::from_json(bytes)?;
    Ok(doc
        .entries
        .into_iter()
        .map(|(key, entry)| {
            (
                key,
                (
                    DocEntry {
                        value: entry.value,
                        encoding: entry.encoding,
                        env: entry.env,
                        description: entry.description,
                    },
                    Some(entry.created_at),
                ),
            )
        })
        .collect())
}

/// `--format dotenv` (3.5.11, Step 3.5): parses `NAME=value`,
/// `NAME="value"`, and `NAME='value'` lines, with an optional leading
/// `export ` kept for shell `.env` files. Blank lines and `#` comment
/// lines are ignored. The key is `NAME` lowercased; `env` is set to
/// `NAME` unchanged.
fn parse_dotenv(bytes: &[u8]) -> anyhow::Result<Imported> {
    let text = std::str::from_utf8(bytes).context("dotenv input is not valid UTF-8")?;
    let mut result = BTreeMap::new();
    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line).trim_start();
        let (name, raw_value) = line
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("line {}: expected NAME=value", index + 1))?;
        let name = name.trim();
        let value = unescape_dotenv_value(raw_value.trim());
        let key = Key::parse(&name.to_lowercase())
            .with_context(|| format!("line {}: {name}", index + 1))?;
        result.insert(
            key,
            (
                DocEntry {
                    value: Value::from_bytes(value.into_bytes())?,
                    encoding: Encoding::Utf8,
                    env: Some(name.to_owned()),
                    description: None,
                },
                None,
            ),
        );
    }
    Ok(result)
}

/// Strip a dotenv value's surrounding quotes, if any, and undo the
/// `\"`, `\\`, `\n` escaping (mirroring `export --format dotenv`'s
/// escaping, 3.5.11) inside a double-quoted value. A single-quoted or
/// unquoted value is taken literally.
fn unescape_dotenv_value(raw: &str) -> String {
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return unescape_double_quoted(&raw[1..raw.len() - 1]);
    }
    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        return raw[1..raw.len() - 1].to_owned();
    }
    raw.to_owned()
}

/// Undo `\"`, `\\`, `\n`; any other `\x` escape is kept verbatim
/// (backslash included), since 3.5.11 defines only those three.
fn unescape_double_quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('\\') | None => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
        }
    }
    out
}

/// `--format toml`: the `edit` document format (3.5.12), via
/// `crate::document::from_toml`. Carries no timestamps, so every entry
/// gets `created_at` = now on import.
fn parse_toml(bytes: &[u8]) -> anyhow::Result<Imported> {
    let text = std::str::from_utf8(bytes).context("toml input is not valid UTF-8")?;
    Ok(crate::document::from_toml(text)?
        .into_iter()
        .map(|(key, doc)| (key, (doc, None)))
        .collect())
}

/// The `import` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct ImportJson {
    ok: bool,
    added: u64,
    updated: u64,
    skipped: u64,
}

/// Print `import`'s result: a one-line summary on stderr in human mode,
/// or the JSON shape from appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, added: u64, updated: u64, skipped: u64) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            output::info(
                ctx.quiet,
                &format!("imported: {added} added, {updated} updated, {skipped} skipped"),
            );
            Ok(())
        }
        OutputMode::Json => output::json(&ImportJson {
            ok: true,
            added,
            updated,
            skipped,
        }),
    }
}
