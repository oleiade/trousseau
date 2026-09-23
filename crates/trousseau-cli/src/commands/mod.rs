//! One module per subcommand, dispatched from [`dispatch`].

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context as _;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use trousseau::schema::{Encoding, Entry, Key, Store};

use crate::cli::{Cli, Command, RecipientsAction};
use crate::context::Context;
use crate::exit::CliError;

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
/// Returns whatever the selected command's implementation returns.
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

/// Format a timestamp as RFC 3339, shared by every command that reports an
/// entry's `created_at` or `updated_at` (3.1.2).
pub fn format_rfc3339(at: OffsetDateTime) -> anyhow::Result<String> {
    at.format(&Rfc3339).context("formatting a timestamp")
}

/// A `utf8` entry's value as text, for `run`, `env`, and `export
/// --format dotenv`, which only ever handle `utf8` entries. Those bytes
/// are always valid UTF-8 (checked at every write path);
/// `unwrap_or_default` is a defensive fallback, never expected to
/// trigger.
pub fn entry_text(entry: &Entry) -> &str {
    std::str::from_utf8(entry.value.expose()).unwrap_or_default()
}

/// Write `bytes` to `path`, readable by the owner only: mode `0600` on
/// Unix, both when the file is created and when an existing one is
/// overwritten. Windows relies on the user profile's ACLs (3.2). Shared
/// by `get --out`, `export --out` (3.5.5, 3.5.11), and `init`'s
/// identity file (3.5.2).
///
/// # Errors
///
/// Returns [`CliError::OutputExists`] if `path` already exists and
/// `overwrite` is `false`, or an I/O error otherwise.
pub fn write_private_file(path: &Path, bytes: &[u8], overwrite: bool) -> anyhow::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(CliError::OutputExists {
                path: path.to_path_buf(),
            }
            .into());
        }
        Err(err) => return Err(err).with_context(|| format!("writing {}", path.display())),
    };
    // `mode` above only applies to a newly created file; an overwritten
    // one keeps its old mode unless reset here.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("setting permissions on {}", path.display()))?;
    }
    file.write_all(bytes)
        .with_context(|| format!("writing {}", path.display()))
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
