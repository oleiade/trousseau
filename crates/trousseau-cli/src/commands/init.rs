//! `trousseau init` (3.5.2).

use std::path::Path;

use anyhow::Context as _;
use serde::Serialize;

use trousseau::schema::{Store, StoreKind};
use trousseau::store::LockMode;

use crate::cli::InitArgs;
use crate::context::{Context, SealMaterial};
use crate::exit::CliError;
use crate::output::{self, OutputMode};

/// Run `init`.
///
/// # Errors
///
/// Returns [`CliError::StoreExists`] if the target store already
/// exists, [`CliError::Usage`] if `--no-self` leaves no recipient,
/// whatever identity, recipient, or passphrase resolution returns, or
/// whatever [`trousseau::store::save`] returns.
pub fn run(ctx: &Context, args: &InitArgs) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store_for_init();
    let path = resolved.path();

    // The library's `Error::KeyExists` is about an entry's key, not the
    // store file itself: this condition is caught here, before any
    // store is opened, and reported as `CliError::StoreExists` (3.5.2).
    if path.exists() {
        return Err(CliError::StoreExists {
            path: path.to_path_buf(),
        }
        .into());
    }

    Context::ensure_parent_dir(path)?;
    let _lock = ctx.lock(path, LockMode::Exclusive)?;

    // Re-check under the lock: another process may have created the
    // store between the check above and acquiring the lock.
    if path.exists() {
        return Err(CliError::StoreExists {
            path: path.to_path_buf(),
        }
        .into());
    }

    let now = ctx.now();
    let store = if args.target.passphrase {
        Store::new(StoreKind::Passphrase, Vec::new(), now)
    } else {
        let recipients = build_recipients(ctx, args)?;
        Store::new(StoreKind::Recipients, recipients, now)
    };

    let seal_material = if args.target.passphrase {
        SealMaterial::Passphrase(ctx.new_store_passphrase()?)
    } else {
        ctx.seal_for(&store)?
    };
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    report(ctx, path, &store)
}

/// Gather the recipients for a new recipients store: the union of
/// `--recipient`, `--recipients-file`, the caller's own recipient
/// (unless `--no-self`), and, when interactive, an offered SSH key
/// (3.5.2), validated and normalized through
/// [`trousseau::identity::normalize_recipients`].
///
/// # Errors
///
/// Returns [`CliError::Usage`] if the resulting list is empty, or
/// whatever reading `--recipients-file`, generating or reading the
/// default identity, or [`trousseau::identity::normalize_recipients`]
/// returns.
fn build_recipients(ctx: &Context, args: &InitArgs) -> anyhow::Result<Vec<String>> {
    let mut recipients = args.target.recipient.clone();
    if let Some(path) = &args.target.recipients_file {
        recipients.extend(read_recipients_file(path)?);
    }
    if !args.target.no_self {
        recipients.push(own_recipient(ctx)?);
    }
    offer_ssh_recipient(ctx, &mut recipients)?;

    if recipients.is_empty() {
        return Err(CliError::Usage("at least one recipient is required".to_owned()).into());
    }

    Ok(trousseau::identity::normalize_recipients(recipients)?)
}

/// Read a `--recipients-file`: one recipient per line, blank lines and
/// `#` comments ignored (3.5.2).
fn read_recipients_file(path: &Path) -> anyhow::Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading recipients file {}", path.display()))?;
    Ok(contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect())
}

/// The caller's own recipient (3.5.2): read from the default identity
/// file if it already exists, or generated (and written) if it does
/// not.
///
/// # Errors
///
/// Returns an error if the default identity file exists but no
/// recipient could be derived from it, or if generating or writing a
/// new identity file fails.
fn own_recipient(ctx: &Context) -> anyhow::Result<String> {
    let identity_path = ctx.default_identity_path();
    if identity_path.is_file() {
        let recipients = trousseau::identity::own_recipients(std::slice::from_ref(&identity_path));
        return recipients.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!(
                "cannot derive a recipient from the existing identity file {}",
                identity_path.display()
            )
        });
    }

    let generated = trousseau::identity::generate_identity(ctx.now());
    write_identity_file(&identity_path, &generated.identity_file_contents)?;
    output::info(
        ctx.quiet,
        &format!("created identity {}", identity_path.display()),
    );
    output::info(
        ctx.quiet,
        &format!("your recipient: {}", generated.recipient),
    );
    Ok(generated.recipient)
}

/// Write a freshly generated identity file's contents to `path` with
/// mode `0600` (3.2, 3.5.2), creating its parent directory (mode
/// `0700`) first.
#[cfg(unix)]
fn write_identity_file(path: &Path, contents: &secrecy::SecretString) -> anyhow::Result<()> {
    use secrecy::ExposeSecret as _;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    Context::ensure_parent_dir(path)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("creating identity file {}", path.display()))?;
    file.write_all(contents.expose_secret().as_bytes())
        .with_context(|| format!("writing identity file {}", path.display()))
}

/// Write a freshly generated identity file's contents to `path` (3.5.2).
/// Windows relies on the user profile's ACLs (3.2); there is no mode to
/// set.
#[cfg(not(unix))]
fn write_identity_file(path: &Path, contents: &secrecy::SecretString) -> anyhow::Result<()> {
    use secrecy::ExposeSecret as _;

    Context::ensure_parent_dir(path)?;
    std::fs::write(path, contents.expose_secret().as_bytes())
        .with_context(|| format!("writing identity file {}", path.display()))
}

/// If interactive (stdin is a terminal and `--no-input` was not given)
/// and `~/.ssh/id_ed25519.pub` exists and is not already in
/// `recipients`, ask whether to add it (3.5.2).
///
/// # Errors
///
/// Returns an error if the SSH public key file exists but cannot be
/// read, or if the confirmation prompt fails.
fn offer_ssh_recipient(ctx: &Context, recipients: &mut Vec<String>) -> anyhow::Result<()> {
    if !ctx.is_stdin_tty || ctx.no_input {
        return Ok(());
    }
    let ssh_pub_path = ctx.home_dir().join(".ssh").join("id_ed25519.pub");
    if !ssh_pub_path.is_file() {
        return Ok(());
    }
    let contents = std::fs::read_to_string(&ssh_pub_path)
        .with_context(|| format!("reading {}", ssh_pub_path.display()))?;
    let candidate = contents.trim();
    if candidate.is_empty() {
        return Ok(());
    }
    let already_present = recipients
        .iter()
        .any(|existing| trousseau::identity::same_recipient(existing, candidate));
    if already_present {
        return Ok(());
    }

    let question = format!(
        "Also encrypt to your SSH key {}? [y/N]",
        ssh_pub_path.display()
    );
    if ctx.confirm(&question, false)? {
        recipients.push(candidate.to_owned());
    }
    Ok(())
}

/// The `init` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct InitJson<'a> {
    ok: bool,
    path: String,
    kind: &'a str,
    recipients: &'a [String],
}

/// Print `init`'s result: `created <path>` on stderr in human mode, or
/// the JSON shape from appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, path: &Path, store: &Store) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            output::info(ctx.quiet, &format!("created {}", path.display()));
            Ok(())
        }
        OutputMode::Json => output::json(&InitJson {
            ok: true,
            path: path.display().to_string(),
            kind: store_kind_label(store.kind),
            recipients: &store.recipients,
        }),
    }
}

/// The `kind` string for a [`StoreKind`] (3.5.2, appendix 5.1).
const fn store_kind_label(kind: StoreKind) -> &'static str {
    match kind {
        StoreKind::Recipients => "recipients",
        StoreKind::Passphrase => "passphrase",
    }
}
