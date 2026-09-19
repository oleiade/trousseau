//! The resolved run context: directories, configuration, identities,
//! output mode, and the operations (`unlock`, `seal_for`, `lock`) that
//! every command builds on.
//!
//! `Context` is built once in `main.rs` from the parsed [`Cli`] and the
//! process environment. Building it can fail (a malformed configuration
//! file); every other method here maps a library
//! [`trousseau::error::Error`] or a prompt failure into an [`anyhow::Error`]
//! for `main.rs` to report through `exit.rs`.

use std::cell::{Cell, RefCell};
use std::io::IsTerminal as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context as _;
use etcetera::BaseStrategy as _;
use secrecy::SecretString;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use trousseau::schema::{Store, StoreKind};
use trousseau::store::{Locator, LockGuard, LockMode, Resolved};

use crate::cli::Cli;
use crate::config::Config;
use crate::output::OutputMode;
use crate::prompt::{self, CliCallbacks};

/// The environment variable holding a passphrase for a passphrase
/// store (3.3.3).
const TROUSSEAU_PASSPHRASE: &str = "TROUSSEAU_PASSPHRASE";

/// The environment variable overriding the configuration file path
/// (3.4).
const TROUSSEAU_CONFIG: &str = "TROUSSEAU_CONFIG";

/// The default lock wait timeout (3.5.1): 5 seconds.
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(5);

/// Recipients or a passphrase, ready to seal a store with (built by
/// [`Context::seal_for`]).
///
/// This exists because [`trousseau::store::Seal`] borrows its material;
/// `SealMaterial` owns it so it can outlive the call that built it, and
/// [`SealMaterial::as_seal`] borrows back into the shape
/// [`trousseau::store::save`] expects.
pub enum SealMaterial {
    /// Seal to a set of age recipients.
    Recipients(Vec<Box<dyn age::Recipient + Send>>),
    /// Seal to a passphrase.
    Passphrase(SecretString),
}

impl SealMaterial {
    /// Borrow this material as a [`trousseau::store::Seal`].
    #[must_use]
    pub fn as_seal(&self) -> trousseau::store::Seal<'_> {
        match self {
            Self::Recipients(recipients) => trousseau::store::Seal::Recipients(recipients),
            Self::Passphrase(passphrase) => trousseau::store::Seal::Passphrase(passphrase),
        }
    }
}

/// The resolved run context.
// `quiet`, `no_input`, `is_stdin_tty`, and `is_stdout_tty` are four
// independent, separately documented flags mirroring 3.5.1; collapsing
// them into an enum would not describe the run context they capture.
//
// `config` and `data_dir` have no reader yet: `run`, `env`, and
// `migrate` start reading `config`'s `[run]`/`[migrate]` tables in
// later steps, and a command that needs the bare data directory (as
// opposed to `personal_store`, already derived from it) has not landed
// yet.
#[allow(clippy::struct_excessive_bools)]
#[allow(dead_code)]
pub struct Context {
    /// The parsed configuration file.
    pub config: Config,
    /// Human or JSON output.
    pub output: OutputMode,
    /// `--quiet`: suppress informational stderr lines.
    pub quiet: bool,
    /// `--no-input`: never prompt.
    pub no_input: bool,
    /// Whether stdin is a terminal.
    pub is_stdin_tty: bool,
    /// Whether stdout is a terminal.
    pub is_stdout_tty: bool,
    /// `<config_dir>/trousseau`.
    pub config_dir: PathBuf,
    /// `<data_dir>/trousseau`.
    pub data_dir: PathBuf,
    /// `<cache_dir>/trousseau`.
    pub cache_dir: PathBuf,

    cwd: PathBuf,
    home: PathBuf,
    personal_store: PathBuf,
    explicit_store: Option<PathBuf>,
    env_store: Option<PathBuf>,
    global_store: bool,
    identity_paths: Vec<PathBuf>,
    passphrase_file: Option<PathBuf>,
    passphrase_cache: RefCell<Option<SecretString>>,
    env_passphrase_warned: Cell<bool>,
}

impl Context {
    /// Build the context from the parsed command line and the process
    /// environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be located, the
    /// current directory cannot be read, or the configuration file
    /// exists but is malformed.
    pub fn new(cli: &Cli) -> anyhow::Result<Self> {
        let cwd = std::env::current_dir().context("reading the current directory")?;
        let home = etcetera::home_dir().context("locating the home directory")?;
        let strategy =
            etcetera::choose_base_strategy().context("locating the platform directories")?;

        let config_dir = strategy.config_dir().join("trousseau");
        let data_dir = strategy.data_dir().join("trousseau");
        let cache_dir = strategy.cache_dir().join("trousseau");

        let config_path = std::env::var_os(TROUSSEAU_CONFIG)
            .map_or_else(|| config_dir.join("config.toml"), PathBuf::from);
        let config = Config::load(&config_path)?;

        let personal_store = data_dir.join("default.trousseau");
        let default_identity_path = config_dir.join("identity.txt");

        let identity_paths = resolve_identity_paths(
            &cli.global.identity,
            cli.global.identity_file.as_deref(),
            &config,
            &home,
            &default_identity_path,
        );

        let is_stdin_tty = std::io::stdin().is_terminal();
        let is_stdout_tty = std::io::stdout().is_terminal();

        Ok(Self {
            config,
            output: if cli.global.json {
                OutputMode::Json
            } else {
                OutputMode::Human
            },
            quiet: cli.global.quiet,
            no_input: cli.global.no_input,
            is_stdin_tty,
            is_stdout_tty,
            config_dir,
            data_dir,
            cache_dir,
            cwd,
            home,
            personal_store,
            explicit_store: cli.global.store.clone(),
            env_store: None,
            global_store: cli.global.global,
            identity_paths,
            passphrase_file: cli.global.passphrase_file.clone(),
            passphrase_cache: RefCell::new(None),
            env_passphrase_warned: Cell::new(false),
        })
    }

    /// The home directory this context resolved (3.2, 3.3.2).
    #[must_use]
    pub fn home_dir(&self) -> &Path {
        &self.home
    }

    /// The default identity path: `<config_dir>/trousseau/identity.txt`
    /// (3.2, 3.5.2).
    #[must_use]
    pub fn default_identity_path(&self) -> PathBuf {
        self.config_dir.join("identity.txt")
    }

    /// Build a [`Locator`] borrowing this context's resolved inputs
    /// (3.2).
    #[must_use]
    pub fn locator(&self) -> Locator<'_> {
        Locator {
            explicit: self.explicit_store.clone(),
            env: self.env_store.clone(),
            global: self.global_store,
            cwd: &self.cwd,
            personal: &self.personal_store,
        }
    }

    /// Resolve the store path for every command except `init` and
    /// `migrate` (3.2).
    #[must_use]
    pub fn resolve_store(&self) -> Resolved {
        self.locator().resolve()
    }

    /// Resolve the store path for `init` and `migrate`: rule 4 becomes
    /// "the current directory", never walking up (3.2).
    #[must_use]
    pub fn resolve_store_for_init(&self) -> Resolved {
        self.locator().resolve_for_init()
    }

    /// The `age::Callbacks` implementor for this run.
    #[must_use]
    pub const fn callbacks(&self) -> CliCallbacks {
        CliCallbacks::new(self.no_input)
    }

    /// Ask a yes/no question, respecting `--no-input` and a non-terminal
    /// stdin (3.5.1: "stdin not a terminal implies `--no-input` for
    /// confirmations").
    ///
    /// # Errors
    ///
    /// Returns an error if the question could not be answered: refused
    /// automatically because input is unavailable, or a terminal I/O
    /// failure.
    pub fn confirm(&self, question: &str, default: bool) -> anyhow::Result<bool> {
        if self.no_input || !self.is_stdin_tty {
            return Ok(default);
        }
        prompt::confirm(question, default)
    }

    /// Read, classify, and decrypt the store at `path`.
    ///
    /// For a passphrase store, obtains the passphrase per 3.3.3 (the
    /// `--passphrase-file` flag, `TROUSSEAU_PASSPHRASE`, or an
    /// interactive prompt) and caches it for the rest of the command's
    /// duration. For a recipients store, loads every identity in 3.3.2
    /// order and fails with [`trousseau::error::Error::NoIdentity`] if none is
    /// available.
    ///
    /// # Errors
    ///
    /// Returns whatever [`trousseau::store::read_raw`],
    /// [`trousseau::envelope::peek_kind`], identity loading, or
    /// [`trousseau::store::open`] returns.
    pub fn unlock(&self, path: &Path) -> anyhow::Result<Store> {
        let raw = trousseau::store::read_raw(path)?;
        let bytes = match &raw {
            trousseau::store::RawStore::Current(bytes) => bytes,
            trousseau::store::RawStore::Legacy(_) => {
                return Err(trousseau::error::Error::LegacyStore {
                    path: path.to_path_buf(),
                }
                .into());
            }
        };
        let kind = trousseau::envelope::peek_kind(bytes)?;
        let store = match kind {
            trousseau::envelope::EnvelopeKind::Passphrase => {
                let passphrase = self.passphrase()?;
                trousseau::store::open(path, trousseau::store::Unlock::Passphrase(&passphrase))?
            }
            trousseau::envelope::EnvelopeKind::Recipients => {
                let identities =
                    trousseau::identity::load_identities(&self.identity_paths, self.callbacks())?;
                if identities.is_empty() {
                    return Err(trousseau::error::Error::NoIdentity.into());
                }
                trousseau::store::open(path, trousseau::store::Unlock::Identities(&identities))?
            }
        };
        Ok(store)
    }

    /// Build the material needed to re-seal `store`: recipients derived
    /// from `store.recipients` for a recipients store, or the passphrase
    /// cached by (or obtained the same way as) [`Context::unlock`] for a
    /// passphrase store.
    ///
    /// # Errors
    ///
    /// Returns whatever [`trousseau::identity::to_age_recipients`]
    /// returns, or whatever obtaining the passphrase returns.
    pub fn seal_for(&self, store: &Store) -> anyhow::Result<SealMaterial> {
        match store.kind {
            StoreKind::Recipients => {
                let recipients =
                    trousseau::identity::to_age_recipients(&store.recipients, self.callbacks())?;
                Ok(SealMaterial::Recipients(recipients))
            }
            StoreKind::Passphrase => Ok(SealMaterial::Passphrase(self.passphrase()?)),
        }
    }

    /// Ask twice for a brand-new store's passphrase (3.3.3): used by
    /// `init --passphrase` and `rekey --to-passphrase`, never by
    /// [`Context::seal_for`] (which reuses an already-unlocked store's
    /// passphrase instead of asking again).
    ///
    /// # Errors
    ///
    /// Returns an error if `--no-input` is set (mapped to
    /// [`trousseau::error::Error::Unlock`], exit code 4) or the terminal cannot
    /// be read.
    pub fn passphrase_for_new_store(&self) -> anyhow::Result<SecretString> {
        if self.no_input {
            return Err(trousseau::error::Error::Unlock {
                reason: "no passphrase available (--no-input)".to_owned(),
            }
            .into());
        }
        prompt::hidden_confirm("New passphrase: ")
    }

    /// The passphrase for a brand-new passphrase store (`init
    /// --passphrase`, 3.5.2): `--passphrase-file` if given, otherwise
    /// the interactive double prompt from
    /// [`Context::passphrase_for_new_store`].
    ///
    /// Unlike [`Context::passphrase`], this never falls back to
    /// `TROUSSEAU_PASSPHRASE` or a single prompt: a brand-new store's
    /// passphrase is either handed over explicitly through a file, or
    /// confirmed interactively (3.3.3).
    ///
    /// # Errors
    ///
    /// Returns an error if `--passphrase-file` was given but cannot be
    /// read, or whatever [`Context::passphrase_for_new_store`] returns.
    pub fn new_store_passphrase(&self) -> anyhow::Result<SecretString> {
        if let Some(path) = &self.passphrase_file {
            return read_passphrase_file(path);
        }
        self.passphrase_for_new_store()
    }

    /// Obtain the store's passphrase, per 3.3.3, caching it for the
    /// rest of the command's duration.
    fn passphrase(&self) -> anyhow::Result<SecretString> {
        if let Some(cached) = self.passphrase_cache.borrow().as_ref() {
            return Ok(cached.clone());
        }
        let passphrase = self.resolve_passphrase()?;
        *self.passphrase_cache.borrow_mut() = Some(passphrase.clone());
        Ok(passphrase)
    }

    /// The 3.3.3 passphrase source order, not consulting the cache.
    fn resolve_passphrase(&self) -> anyhow::Result<SecretString> {
        if let Some(path) = &self.passphrase_file {
            return read_passphrase_file(path);
        }
        if let Ok(value) = std::env::var(TROUSSEAU_PASSPHRASE) {
            if self.is_stdin_tty && !self.env_passphrase_warned.replace(true) {
                crate::output::warn(
                    "warning: TROUSSEAU_PASSPHRASE is set; prefer --passphrase-file or an identity",
                );
            }
            return Ok(SecretString::from(value));
        }
        if self.no_input || !self.is_stdin_tty {
            return Err(trousseau::error::Error::Unlock {
                reason: "no passphrase available (--no-input)".to_owned(),
            }
            .into());
        }
        prompt::hidden("Passphrase: ")
    }

    /// Acquire an advisory lock on `path` (3.2, 3.5.1): 5 seconds by
    /// default, overridable in debug builds by
    /// `TROUSSEAU_TEST_LOCK_TIMEOUT_MS` (appendix 5.2).
    ///
    /// # Errors
    ///
    /// Returns [`trousseau::error::Error::LockTimeout`] if the lock is still
    /// held when the timeout elapses, or an I/O error.
    pub fn lock(&self, path: &Path, mode: LockMode) -> anyhow::Result<LockGuard> {
        let lock_dir = self.cache_dir.join("locks");
        let guard = trousseau::store::lock(path, &lock_dir, mode, self.lock_timeout())?;
        Ok(guard)
    }

    /// The lock wait timeout: 5 seconds, or
    /// `TROUSSEAU_TEST_LOCK_TIMEOUT_MS` in debug builds (appendix 5.2).
    // `&self` is part of this method's shape deliberately (it belongs
    // next to `Context::lock`, and a future step may read a configured
    // timeout from `self.config`), even though today it reads no field.
    #[allow(clippy::unused_self)]
    fn lock_timeout(&self) -> Duration {
        #[cfg(debug_assertions)]
        if let Ok(raw) = std::env::var("TROUSSEAU_TEST_LOCK_TIMEOUT_MS")
            && let Ok(ms) = raw.parse::<u64>()
        {
            return Duration::from_millis(ms);
        }
        DEFAULT_LOCK_TIMEOUT
    }

    /// The current time: `OffsetDateTime::now_utc()`, or
    /// `TROUSSEAU_TEST_NOW` in debug builds (appendix 5.2).
    // `&self`, deliberately, so every command reads "now" through the
    // same `Context` its other state comes from, even though today it
    // reads no field.
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn now(&self) -> OffsetDateTime {
        #[cfg(debug_assertions)]
        if let Ok(raw) = std::env::var("TROUSSEAU_TEST_NOW")
            && let Ok(parsed) = OffsetDateTime::parse(&raw, &Rfc3339)
        {
            return parsed;
        }
        OffsetDateTime::now_utc()
    }

    /// Create `path`'s parent directory (and every ancestor above it)
    /// if it does not exist yet, setting its permissions to `0700` on
    /// Unix. Used by `init` before writing the default identity file or
    /// a new store.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
        let Some(parent) = path.parent() else {
            return Ok(());
        };
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                .with_context(|| format!("setting permissions on {}", parent.display()))?;
        }
        Ok(())
    }
}

/// Read a passphrase file's content, stripping exactly one trailing
/// `\r\n` or `\n` (3.3.3). Shared by [`Context::resolve_passphrase`] and
/// [`Context::new_store_passphrase`].
fn read_passphrase_file(path: &Path) -> anyhow::Result<SecretString> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading passphrase file {}", path.display()))?;
    let trimmed = contents
        .strip_suffix("\r\n")
        .or_else(|| contents.strip_suffix('\n'))
        .unwrap_or(&contents);
    Ok(SecretString::from(trimmed.to_owned()))
}

/// Build the 3.3.2 identity path list, in order, filtered to paths that
/// currently exist on disk.
fn resolve_identity_paths(
    cli_identity: &[PathBuf],
    identity_file_env: Option<&Path>,
    config: &Config,
    home: &Path,
    default_identity_path: &Path,
) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.extend(cli_identity.iter().cloned());
    if let Some(path) = identity_file_env {
        candidates.push(path.to_path_buf());
    }
    for raw in &config.identity.files {
        candidates.push(crate::config::expand_tilde(raw, home));
    }
    candidates.push(default_identity_path.to_path_buf());
    candidates.push(home.join(".ssh/id_ed25519"));
    candidates.push(home.join(".ssh/id_rsa"));

    candidates
        .into_iter()
        .filter(|path| std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file()))
        .collect()
}

// step 3.4
impl Context {
    /// The resolved 3.3.2 identity path list (existing files only).
    ///
    /// Used by `recipients rm` to check whether a recipient being
    /// removed is one of the caller's own, through
    /// [`trousseau::identity::own_recipients`] (3.5.9).
    #[must_use]
    pub fn identity_paths(&self) -> &[PathBuf] {
        &self.identity_paths
    }
}
