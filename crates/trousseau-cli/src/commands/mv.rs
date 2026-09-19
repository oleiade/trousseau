//! `trousseau mv` (3.5.8).

use serde::Serialize;

use trousseau::schema::Key;
use trousseau::store::LockMode;

use crate::cli::MvArgs;
use crate::context::Context;
use crate::output::{self, OutputMode};

/// Run `mv`.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::InvalidKey`] if `OLD` or `NEW` does
/// not satisfy the key grammar, [`trousseau::error::Error::KeyNotFound`]
/// if `OLD` does not exist, [`trousseau::error::Error::KeyExists`] if
/// `NEW` already exists and `--force` is not given, or whatever
/// [`Context::unlock`], [`Context::seal_for`], or
/// [`trousseau::store::save`] returns.
pub fn run(ctx: &Context, args: &MvArgs) -> anyhow::Result<()> {
    let old = Key::parse(&args.old)?;
    let new = Key::parse(&args.new)?;

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    let mut store = ctx.unlock(path)?;
    let now = ctx.now();
    store.rename(&old, new.clone(), args.force, now)?;

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    report(ctx, &old, &new)
}

/// The `mv` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct MvJson<'a> {
    ok: bool,
    from: &'a str,
    to: &'a str,
}

/// Print `mv`'s result: `moved OLD -> NEW` on stderr in human mode, or
/// the JSON shape from appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, old: &Key, new: &Key) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            output::info(ctx.quiet, &format!("moved {old} -> {new}"));
            Ok(())
        }
        OutputMode::Json => output::json(&MvJson {
            ok: true,
            from: old.as_str(),
            to: new.as_str(),
        }),
    }
}
