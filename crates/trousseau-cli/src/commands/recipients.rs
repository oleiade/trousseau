//! `trousseau recipients ls|add|rm` (3.5.9).

use std::path::Path;

use serde::Serialize;

use trousseau::envelope::EnvelopeKind;
use trousseau::store::LockMode;

use crate::context::Context;
use crate::exit::CliError;
use crate::output::{self, OutputMode};

/// Run `recipients ls`.
///
/// # Errors
///
/// Returns an error (exit 1) if the store is a passphrase store, or
/// whatever [`Context::unlock`] returns.
pub fn run_ls(ctx: &Context) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Shared)?;

    require_recipients_store(path)?;
    let store = ctx.unlock(path)?;

    report_ls(ctx, &store.recipients)
}

/// Run `recipients add`.
///
/// # Errors
///
/// Returns an error (exit 1) if the store is a passphrase store,
/// [`trousseau::error::Error::InvalidRecipient`] if any of `recipients`
/// does not parse (3.3.1), or whatever [`Context::unlock`],
/// [`Context::seal_for`], or [`trousseau::store::save`] returns.
pub fn run_add(ctx: &Context, recipients: &[String]) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    require_recipients_store(path)?;
    let mut store = ctx.unlock(path)?;

    // Validate every recipient before changing anything: an invalid
    // recipient aborts the whole command (3.3.1), whether or not it
    // would also have been a duplicate.
    for raw in recipients {
        trousseau::identity::parse_recipient(raw)?;
    }

    let mut combined = store.recipients.clone();
    let mut added = Vec::new();
    for raw in recipients {
        let already_present = store
            .recipients
            .iter()
            .any(|existing| trousseau::identity::same_recipient(existing, raw));
        if already_present {
            output::warn(&format!("recipient already present, ignored: {raw}"));
            continue;
        }
        combined.push(raw.clone());
        added.push(raw.clone());
    }
    store.recipients = trousseau::identity::normalize_recipients(combined)?;

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    for raw in &added {
        output::info(ctx.quiet, &format!("added {raw}"));
    }
    report_mutation(ctx, &store.recipients)
}

/// Run `recipients rm`.
///
/// # Errors
///
/// Returns an error (exit 1) if the store is a passphrase store,
/// [`trousseau::error::Error::InvalidRecipient`] if any of `recipients`
/// does not parse, [`CliError::Usage`] (exit 2) if removing them would
/// leave the store with no recipients, [`CliError::Refused`] (exit 2)
/// if one of the caller's own recipients would be removed and neither
/// an interactive confirmation nor `force` allowed it, or whatever
/// [`Context::unlock`], [`Context::seal_for`], or
/// [`trousseau::store::save`] returns.
pub fn run_rm(ctx: &Context, recipients: &[String], force: bool) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    require_recipients_store(path)?;
    let mut store = ctx.unlock(path)?;

    for raw in recipients {
        trousseau::identity::parse_recipient(raw)?;
    }

    let mut removed = Vec::new();
    let mut remaining = Vec::new();
    for existing in &store.recipients {
        let matched = recipients
            .iter()
            .any(|target| trousseau::identity::same_recipient(existing, target));
        if matched {
            removed.push(existing.clone());
        } else {
            remaining.push(existing.clone());
        }
    }

    if remaining.is_empty() {
        return Err(CliError::Usage("refusing to remove the last recipient".to_owned()).into());
    }

    guard_own_recipient(ctx, &removed, force)?;

    store.recipients = remaining;

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    for raw in &removed {
        output::info(ctx.quiet, &format!("removed {raw}"));
    }
    report_mutation(ctx, &store.recipients)
}

/// If any of `removed` matches one of the caller's own identities
/// (3.3.2, via [`trousseau::identity::own_recipients`]), print the
/// warning 3.5.9 requires and require either `force` or an interactive
/// confirmation.
///
/// # Errors
///
/// Returns [`CliError::Refused`] if the removal is declined: `force`
/// was not given and either the confirmation was answered no, or no
/// confirmation could be asked at all (`--no-input` or a non-terminal
/// stdin, 3.5.1).
fn guard_own_recipient(ctx: &Context, removed: &[String], force: bool) -> anyhow::Result<()> {
    let own = trousseau::identity::own_recipients(ctx.identity_paths());
    let removes_own = removed.iter().any(|r| {
        own.iter()
            .any(|o| trousseau::identity::same_recipient(o, r))
    });
    if !removes_own {
        return Ok(());
    }

    output::warn(
        "warning: you removed your own recipient; you will not be able to open this store after this command",
    );
    if force {
        return Ok(());
    }
    if ctx.confirm("Remove your own recipient?", false)? {
        Ok(())
    } else {
        Err(CliError::Refused.into())
    }
}

/// Return an error if the store at `path` is a passphrase store: 3.5.9
/// restricts `recipients` to recipients stores.
///
/// Uses [`Context::peek_kind`] rather than [`Context::unlock`], so this
/// fails before ever asking for a passphrase that would just be
/// rejected a moment later.
fn require_recipients_store(path: &Path) -> anyhow::Result<()> {
    if Context::peek_kind(path)? == EnvelopeKind::Passphrase {
        return Err(anyhow::anyhow!(
            "this is a passphrase store; use 'rekey --to-recipients'"
        ));
    }
    Ok(())
}

/// The `recipients ls` output shape: one recipient per line as stored
/// in human mode, or a plain array of strings in `--json` mode
/// (appendix 5.1).
fn report_ls(ctx: &Context, recipients: &[String]) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            for recipient in recipients {
                output::line(recipient);
            }
            Ok(())
        }
        OutputMode::Json => output::json(&recipients),
    }
}

/// The `recipients add`/`recipients rm` `--json` output shape (appendix
/// 5.1): the human-readable `added `/`removed ` lines are printed by
/// the caller before this runs.
#[derive(Serialize)]
struct RecipientsJson<'a> {
    ok: bool,
    recipients: &'a [String],
}

/// Print the JSON shape shared by `recipients add` and `recipients rm`;
/// a no-op in human mode, since the per-recipient `added `/`removed `
/// lines are already printed by the caller.
fn report_mutation(ctx: &Context, recipients: &[String]) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => Ok(()),
        OutputMode::Json => output::json(&RecipientsJson {
            ok: true,
            recipients,
        }),
    }
}
