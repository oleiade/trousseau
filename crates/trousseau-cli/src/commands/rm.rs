//! `trousseau rm` (3.5.7).

use serde::Serialize;

use trousseau::error::Error;
use trousseau::schema::Key;
use trousseau::store::LockMode;

use crate::cli::RmArgs;
use crate::context::Context;
use crate::output::{self, OutputMode};

/// Run `rm`.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::InvalidKey`] if any `KEY` does not
/// satisfy the key grammar, [`trousseau::error::Error::KeyNotFound`] for
/// the first missing key when `--force` is not given (nothing is written
/// in that case: every key is checked before any is removed), or
/// whatever [`Context::unlock`], [`Context::seal_for`], or
/// [`trousseau::store::save`] returns.
pub fn run(ctx: &Context, args: &RmArgs) -> anyhow::Result<()> {
    let keys = args
        .keys
        .iter()
        .map(|raw| Key::parse(raw))
        .collect::<Result<Vec<_>, _>>()?;

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    let mut store = ctx.unlock(path)?;

    if !args.force {
        for key in &keys {
            if !store.entries.contains_key(key) {
                return Err(Error::KeyNotFound {
                    key: key.to_string(),
                }
                .into());
            }
        }
    }

    let now = ctx.now();
    let mut removed = Vec::new();
    for key in &keys {
        if store.remove(key, now).is_some() {
            removed.push(key.clone());
        }
    }

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    report(ctx, &removed)
}

/// The `rm` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct RmJson {
    ok: bool,
    removed: Vec<String>,
}

/// Print `rm`'s result: `removed KEY` per removed key on stderr in human
/// mode, or the JSON shape from appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, removed: &[Key]) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            for key in removed {
                output::info(ctx.quiet, &format!("removed {key}"));
            }
            Ok(())
        }
        OutputMode::Json => output::json(&RmJson {
            ok: true,
            removed: removed.iter().map(ToString::to_string).collect(),
        }),
    }
}
