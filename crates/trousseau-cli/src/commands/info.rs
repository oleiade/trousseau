//! `trousseau info` (3.5.3).

use std::path::Path;

use anyhow::Context as _;
use serde::Serialize;
use time::format_description::well_known::Rfc3339;

use trousseau::envelope::EnvelopeKind;
use trousseau::schema::Store;
use trousseau::store::LockMode;

use crate::context::Context;
use crate::output::{self, OutputMode};

/// The label column width used by [`print_human`]: the longest label,
/// `"recipients:"` (11 bytes), plus a two-space gap (3.5.3).
const LABEL_WIDTH: usize = 13;

/// Run `info`.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::StoreNotFound`],
/// [`trousseau::error::Error::LegacyStore`], or
/// [`trousseau::error::Error::InvalidStore`] if the store cannot even be
/// classified. If it can be classified but not unlocked (wrong
/// passphrase, no matching identity), prints the `(locked)` form and
/// returns `Ok`, per 3.5.3.
pub fn run(ctx: &Context) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Shared)?;

    // A legacy store is never reported as "(locked)": `peek_kind` fails
    // with exit 7 and the `migrate` hint, same as every other command
    // (3.5.3, 3.6).
    let kind = Context::peek_kind(path)?;

    match ctx.unlock(path) {
        Ok(store) => report(ctx, path, &Unlocked::Store(&store)),
        Err(err) if is_unlock_failure(&err) => report(ctx, path, &Unlocked::Locked(kind)),
        Err(err) => Err(err),
    }
}

/// `true` if `err` is the kind of failure 3.5.3 calls "unlocking
/// fails": exactly the [`trousseau::error::Error`] variants `exit.rs`
/// maps to exit code 4 (cannot unlock: no identity, wrong passphrase, no
/// matching identity).
fn is_unlock_failure(err: &anyhow::Error) -> bool {
    crate::exit::code_for(err) == 4
}

/// What [`run`] learned about the store: either it unlocked
/// successfully, or only its envelope `kind` is known.
enum Unlocked<'a> {
    Store(&'a Store),
    Locked(EnvelopeKind),
}

/// Print `info`'s result: the aligned human table from 3.5.3, or the
/// JSON shape from appendix 5.1.
fn report(ctx: &Context, path: &Path, unlocked: &Unlocked<'_>) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => print_human(path, unlocked),
        OutputMode::Json => print_json(path, unlocked),
    }
}

fn print_human(path: &Path, unlocked: &Unlocked<'_>) -> anyhow::Result<()> {
    let mut lines = vec![
        format!("{:<LABEL_WIDTH$}{}", "path:", path.display()),
        format!("{:<LABEL_WIDTH$}{}", "kind:", kind_label(unlocked)),
    ];
    match unlocked {
        Unlocked::Store(store) => {
            let updated = store
                .updated_at
                .format(&Rfc3339)
                .context("formatting the store's updated_at")?;
            lines.push(format!("{:<LABEL_WIDTH$}{}", "schema:", store.schema));
            lines.push(format!(
                "{:<LABEL_WIDTH$}{}",
                "recipients:",
                store.recipients.len()
            ));
            lines.push(format!(
                "{:<LABEL_WIDTH$}{}",
                "entries:",
                store.entries.len()
            ));
            lines.push(format!("{:<LABEL_WIDTH$}{updated}", "updated:"));
        }
        Unlocked::Locked(_) => {
            for label in ["schema:", "recipients:", "entries:", "updated:"] {
                lines.push(format!("{label:<LABEL_WIDTH$}(locked)"));
            }
        }
    }
    for line in &lines {
        output::line(line);
    }
    Ok(())
}

/// The `info` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct InfoJson {
    path: String,
    kind: &'static str,
    schema: Option<u32>,
    recipients: Option<usize>,
    entries: Option<usize>,
    updated_at: Option<String>,
    locked: bool,
}

fn print_json(path: &Path, unlocked: &Unlocked<'_>) -> anyhow::Result<()> {
    let json = match unlocked {
        Unlocked::Store(store) => {
            let updated_at = store
                .updated_at
                .format(&Rfc3339)
                .context("formatting the store's updated_at")?;
            InfoJson {
                path: path.display().to_string(),
                kind: kind_label(unlocked),
                schema: Some(store.schema),
                recipients: Some(store.recipients.len()),
                entries: Some(store.entries.len()),
                updated_at: Some(updated_at),
                locked: false,
            }
        }
        Unlocked::Locked(_) => InfoJson {
            path: path.display().to_string(),
            kind: kind_label(unlocked),
            schema: None,
            recipients: None,
            entries: None,
            updated_at: None,
            locked: true,
        },
    };
    output::json(&json)
}

/// The `kind` string shared by both output modes (3.5.3, appendix 5.1):
/// [`trousseau::schema::StoreKind::as_str`] once unlocked,
/// [`EnvelopeKind::as_str`] while still locked. The two agree on every
/// variant (see [`EnvelopeKind::as_str`]'s doc comment).
const fn kind_label(unlocked: &Unlocked<'_>) -> &'static str {
    match unlocked {
        Unlocked::Store(store) => store.kind.as_str(),
        Unlocked::Locked(kind) => kind.as_str(),
    }
}
