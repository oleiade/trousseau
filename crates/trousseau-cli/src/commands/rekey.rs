//! `trousseau rekey` (3.5.10).

use serde::Serialize;

use trousseau::schema::{Store, StoreKind};
use trousseau::store::LockMode;

use crate::cli::RekeyArgs;
use crate::context::{Context, SealMaterial};
use crate::output::{self, OutputMode};

/// Run `rekey`.
///
/// The store is fully unlocked, and the new seal material built
/// entirely in memory, before [`trousseau::store::save`] ever touches
/// disk: nothing is written unless both the unlock and the reseal
/// succeed (review focus, step 3.4).
///
/// # Errors
///
/// Returns whatever [`Context::unlock`] returns. For `--to-passphrase`,
/// whatever [`Context::new_store_passphrase`] returns. For
/// `--to-recipients`, [`trousseau::error::Error::InvalidRecipient`] if
/// any of the given recipients does not parse (3.3.1). Otherwise,
/// whatever [`Context::seal_for`] or [`trousseau::store::save`]
/// returns.
pub fn run(ctx: &Context, args: &RekeyArgs) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    let mut store = ctx.unlock(path)?;

    let seal_material = if args.to_passphrase {
        // The double-prompt-or-`--passphrase-file` flow for a brand-new
        // passphrase, same as `init --passphrase` (3.3.3): never
        // `Context::seal_for`, which reuses an already-known passphrase
        // instead of establishing a new one.
        let passphrase = ctx.new_store_passphrase()?;
        store.kind = StoreKind::Passphrase;
        store.recipients = Vec::new();
        SealMaterial::Passphrase(passphrase)
    } else if let Some(recipients) = &args.to_recipients {
        // Exactly the listed recipients: the caller's own recipient is
        // not added implicitly (3.5.10).
        store.kind = StoreKind::Recipients;
        store.recipients = trousseau::identity::normalize_recipients(recipients.clone())?;
        ctx.seal_for(&store)?
    } else {
        // No flags: re-encrypt to the current kind and recipients (or
        // passphrase). `store::save` always mints a fresh file key, so
        // this alone changes the ciphertext (3.5.10).
        ctx.seal_for(&store)?
    };

    trousseau::store::save(path, &store, seal_material.as_seal())?;

    report(ctx, &store)
}

/// The `rekey` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct RekeyJson<'a> {
    ok: bool,
    kind: &'a str,
    recipients: &'a [String],
}

/// Print `rekey`'s result: a one-line summary on stderr in human mode,
/// or the JSON shape from appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, store: &Store) -> anyhow::Result<()> {
    let kind = store_kind_label(store.kind);
    match ctx.output {
        OutputMode::Human => {
            output::info(ctx.quiet, &format!("rekeyed ({kind})"));
            Ok(())
        }
        OutputMode::Json => output::json(&RekeyJson {
            ok: true,
            kind,
            recipients: &store.recipients,
        }),
    }
}

/// The `kind` string for a [`StoreKind`] (appendix 5.1). A small,
/// deliberate duplicate of `commands::init`'s own private helper of the
/// same shape, rather than a shared one, per step 3.4's minimal-touch
/// note.
const fn store_kind_label(kind: StoreKind) -> &'static str {
    match kind {
        StoreKind::Recipients => "recipients",
        StoreKind::Passphrase => "passphrase",
    }
}
