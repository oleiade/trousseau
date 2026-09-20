//! One module per subcommand, dispatched from [`dispatch`].
//!
//! Every command not yet implemented (everything before its plan step
//! lands) returns `anyhow::anyhow!("not implemented yet (step N)")` with
//! the step from `docs/IMPLEMENTATION_PLAN.md` section 4 that implements
//! it. `main.rs` maps that generic error to exit code 1 through
//! `exit.rs`, same as any other unexpected failure.

use std::collections::BTreeMap;

use anyhow::Context as _;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use trousseau::schema::{Encoding, Entry, Key, Store};

use crate::cli::{Cli, Command, RecipientsAction};
use crate::context::Context;

pub mod clip;
pub mod completions;
pub mod edit;
pub mod env;
pub mod export;
pub mod get;
pub mod import;
pub mod info;
pub mod init;
pub mod ls;
pub mod man;
pub mod migrate;
pub mod mv;
pub mod recipients;
pub mod rekey;
pub mod rm;
pub mod run;
pub mod set;

/// Run whichever subcommand `cli` selected.
///
/// # Errors
///
/// Returns whatever the selected command's implementation returns; for
/// a command not yet implemented, a generic "not implemented yet"
/// error.
pub fn dispatch(cli: &Cli, ctx: &Context) -> anyhow::Result<()> {
    match &cli.command {
        Command::Init(args) => init::run(ctx, args),
        Command::Info => info::run(ctx),
        Command::Set(args) => set::run(ctx, args),
        Command::Get(args) => get::run(ctx, args),
        Command::Ls(args) => ls::run(ctx, args),
        Command::Rm(args) => rm::run(ctx, args),
        Command::Mv(args) => mv::run(ctx, args),
        Command::Recipients { action } => match action {
            RecipientsAction::Ls => recipients::run_ls(ctx),
            RecipientsAction::Add { recipients } => recipients::run_add(ctx, recipients),
            RecipientsAction::Rm { recipients, force } => {
                recipients::run_rm(ctx, recipients, *force)
            }
        },
        Command::Rekey(args) => rekey::run(ctx, args),
        Command::Export(args) => export::run(ctx, args),
        Command::Import(args) => import::run(ctx, args),
        Command::Run(args) => run::run(ctx, args),
        Command::Env(args) => env::run(ctx, args),
        Command::Edit => edit::run(ctx),
        Command::Migrate(args) => migrate::run(ctx, args),
        Command::Completions(args) => completions::run(args),
        Command::Man(args) => man::run(args),
        Command::ClipClear(args) => clip::run(ctx, args),
        #[cfg(debug_assertions)]
        Command::PanicTest { .. } => panic_test(),
    }
}

/// `__panic-test` (hidden, debug builds only): panic on purpose so a
/// test can exercise `main.rs`'s panic hook end to end.
///
/// This is the one place in the crate allowed to panic outside tests:
/// it exists solely to trigger the panic hook, so the crate-level
/// `clippy::panic` deny is overridden here, and only here.
#[cfg(debug_assertions)]
#[allow(clippy::panic)]
fn panic_test() -> anyhow::Result<()> {
    panic!("panic test")
}

/// The `encoding` string shared by `set`, `get`, and `ls` (3.1.2, appendix
/// 5.1).
pub const fn encoding_label(encoding: Encoding) -> &'static str {
    match encoding {
        Encoding::Utf8 => "utf8",
        Encoding::Base64 => "base64",
    }
}

/// Format a timestamp as RFC 3339, shared by every command that reports an
/// entry's `created_at` or `updated_at` (3.1.2).
pub fn format_rfc3339(at: OffsetDateTime) -> anyhow::Result<String> {
    at.format(&Rfc3339).context("formatting a timestamp")
}

/// The selection, skipping, and conflict rules shared by `run` and `env`
/// (3.5.13, 3.5.14).
///
/// Parses `only_raw`'s path prefixes (same grammar and semantics as `ls
/// PREFIX`), prints `skipping binary entry KEY` to stderr for every
/// `base64` entry the selection would otherwise have included (in key
/// order, before the environment mapping is built), then delegates to
/// [`Store::env_map`] for the prefix, filtering, and conflict-detection
/// rules of 3.1.4.
///
/// # Errors
///
/// Returns [`trousseau::error::Error::InvalidKey`] if a `--only` value
/// does not satisfy the key grammar, [`trousseau::error::Error::InvalidStore`]
/// if `prefix` does not match the environment-name grammar (3.1.4), or
/// [`trousseau::error::Error::EnvConflict`] if two selected entries resolve
/// to the same environment variable name.
pub fn resolve_env_selection<'a>(
    store: &'a Store,
    prefix: &str,
    only_raw: &[String],
) -> anyhow::Result<BTreeMap<String, &'a Entry>> {
    let only: Vec<Key> = only_raw
        .iter()
        .map(|p| Key::parse(p))
        .collect::<Result<_, _>>()?;

    for (key, entry) in &store.entries {
        if matches!(entry.encoding, Encoding::Base64)
            && (only.is_empty() || only.iter().any(|p| key.has_path_prefix(p)))
        {
            crate::output::warn(&format!("skipping binary entry {key}"));
        }
    }

    Ok(store.env_map(prefix, &only)?)
}
