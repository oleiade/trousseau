//! The `clap` command tree.
//!
//! Every subcommand and flag from `docs/IMPLEMENTATION_PLAN.md` section 3.5
//! is declared here, even the ones not implemented until a later step (see
//! `crate::commands`), so `--help` is complete and stable from this step
//! on. This module builds argument structs only: it has no I/O and knows
//! nothing about the store format, `trousseau::error::Error`, or output modes.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// `trousseau`: a portable, encrypted keyring for storing and sharing
/// secrets from the command line.
#[derive(Debug, Parser)]
#[command(
    name = "trousseau",
    bin_name = "trousseau",
    version,
    about,
    propagate_version = false,
    after_help = crate::help::TOP,
    after_long_help = crate::help::TOP_LONG
)]
pub struct Cli {
    /// Flags valid before or after the subcommand (3.5.1).
    #[command(flatten)]
    pub global: GlobalArgs,

    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Global flags (3.5.1): valid before or after the subcommand.
// Four independent flags (`--global`, `--json`, `--quiet`,
// `--no-input`), each with its own documented meaning per 3.5.1;
// collapsing them into an enum would not describe the CLI grammar
// they mirror.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Args)]
#[command(next_help_heading = "Global options")]
pub struct GlobalArgs {
    /// Use this store file. Wins over `--global` and over store
    /// discovery.
    #[arg(long, global = true, env = "TROUSSEAU_STORE", value_name = "PATH")]
    pub store: Option<PathBuf>,

    /// Use your personal store instead of the nearest project store (the
    /// `.trousseau` file in this directory or a parent).
    #[arg(long, global = true)]
    pub global: bool,

    /// An identity (private key) file to try when unlocking. Repeatable.
    /// Also tried: `TROUSSEAU_IDENTITY_FILE`, the config file's
    /// `identity.files`, the default identity file, `~/.ssh/id_ed25519`
    /// and `~/.ssh/id_rsa`.
    #[arg(long = "identity", global = true, value_name = "PATH")]
    pub identity: Vec<PathBuf>,

    /// Hidden, single-value counterpart to `--identity` populated from
    /// `TROUSSEAU_IDENTITY_FILE` (3.3.2). Not meant to be typed directly;
    /// it exists so the environment variable has a place to land in the
    /// parsed arguments. Merged into the identity list by `context.rs`.
    #[arg(
        long = "identity-file",
        hide = true,
        global = true,
        env = "TROUSSEAU_IDENTITY_FILE",
        value_name = "PATH"
    )]
    pub identity_file: Option<PathBuf>,

    /// Read the store's passphrase from this file instead of prompting.
    /// One trailing newline is stripped.
    #[arg(long = "passphrase-file", global = true, value_name = "PATH")]
    pub passphrase_file: Option<PathBuf>,

    /// Print one JSON document on stdout instead of text. Errors become
    /// a JSON object on stderr.
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress informational messages on stderr. Errors still print.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Never prompt. A command that would prompt fails instead: exit 4
    /// for a secret, exit 2 for a confirmation.
    #[arg(long = "no-input", global = true)]
    pub no_input: bool,
}

/// Every `trousseau` subcommand (3.5).
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new, empty store.
    ///
    /// Without flags, creates `.trousseau` in the current directory,
    /// encrypted to your own key. If you have no identity yet, `init`
    /// generates one and prints your recipient (your public key,
    /// `age1...`). Share that recipient with teammates so they can add
    /// you to their stores.
    #[command(after_help = crate::help::INIT)]
    Init(InitArgs),

    /// Show which store is in use, and what is in it.
    ///
    /// Prints the store's path, kind (recipients or passphrase), schema
    /// version, and recipient and entry counts. Works even when the
    /// store cannot be unlocked: the path and kind still print.
    #[command(after_help = crate::help::INFO)]
    Info,

    /// Save a value under a key.
    ///
    /// The value is never passed on the command line, where it would
    /// end up in your shell history and the process list. With no
    /// flags, `set` prompts for it (hidden input), or reads it from
    /// stdin when stdin is piped.
    #[command(after_help = crate::help::SET)]
    Set(SetArgs),

    /// Print a key's value.
    ///
    /// Writes the raw value to stdout. On a terminal a newline is added
    /// after text values, and binary values are refused (use `--out` or
    /// a pipe). When piped, the bytes are written exactly as stored.
    #[command(after_help = crate::help::GET)]
    Get(GetArgs),

    /// List keys. Never prints values.
    #[command(after_help = crate::help::LS)]
    Ls(LsArgs),

    /// Remove one or more keys.
    ///
    /// All or nothing: if any key is missing, nothing is removed (exit
    /// 5), unless `--force` is given.
    #[command(after_help = crate::help::RM)]
    Rm(RmArgs),

    /// Rename a key, keeping its value and metadata.
    #[command(after_help = crate::help::MV)]
    Mv(MvArgs),

    /// Manage who can open a recipients store.
    ///
    /// A recipient is a public key. Everyone whose recipient is in the
    /// list can decrypt the store with their matching identity (private
    /// key). Accepted forms: an age key (`age1...`), an SSH public key
    /// (`ssh-ed25519 ...` or `ssh-rsa ...`), or an age plugin recipient
    /// (such as `age1yubikey1...`).
    #[command(after_help = crate::help::RECIPIENTS)]
    Recipients {
        /// The recipients operation to perform.
        #[command(subcommand)]
        action: RecipientsAction,
    },

    /// Re-encrypt the store, optionally changing how it is protected.
    ///
    /// With no flags, re-encrypts to the same recipients (or
    /// passphrase) with a fresh file key. The entries themselves do not
    /// change.
    #[command(after_help = crate::help::REKEY)]
    Rekey(RekeyArgs),

    /// Write the store's entries out in plaintext.
    ///
    /// The output is NOT encrypted. Prefer `run` or `env` to hand
    /// secrets to a program, and delete exported files when you are
    /// done.
    #[command(after_help = crate::help::EXPORT)]
    Export(ExportArgs),

    /// Add entries from a JSON, dotenv, or TOML document.
    ///
    /// Reads the file given as `PATH`, or stdin when `PATH` is omitted.
    /// Only entries are merged: the imported document's recipients and
    /// kind are ignored.
    #[command(after_help = crate::help::IMPORT)]
    Import(ImportArgs),

    /// Run a command with the store's entries as environment variables.
    ///
    /// Decrypts the store, adds one variable per entry to the
    /// environment, and runs `CMD`. Nothing is written to disk. `run`
    /// exits with `CMD`'s exit code. Everything after `--` is the
    /// command and its arguments.
    #[command(after_help = crate::help::RUN)]
    Run(RunArgs),

    /// Print the store's entries as environment variable assignments.
    ///
    /// Same naming and selection rules as `run`. The output contains
    /// secret values in plaintext: send it to `eval` or a pipe, not to
    /// your terminal scrollback or a committed file.
    #[command(after_help = crate::help::ENV)]
    Env(EnvArgs),

    /// Edit the whole store as a TOML document in your editor.
    ///
    /// Opens a temporary plaintext copy in `$VISUAL`, then `$EDITOR`,
    /// then `vi`. Save and quit to apply. Leave the file empty, or quit
    /// without changes, to abort. The temporary file is deleted
    /// afterward in every case.
    #[command(after_help = crate::help::EDIT)]
    Edit,

    /// Copy a legacy v0.4 store into a new store.
    ///
    /// Reads the old file (usually `~/.trousseau`) and creates a new
    /// store, following the same rules and flags as `init`. The old
    /// file is never modified or deleted.
    #[command(after_help = crate::help::MIGRATE)]
    Migrate(MigrateArgs),

    /// Print a shell completion script.
    #[command(after_help = crate::help::COMPLETIONS)]
    Completions(CompletionsArgs),

    /// Print a man page in roff format.
    #[command(after_help = crate::help::MAN)]
    Man(ManArgs),

    /// Clear the clipboard after `get --clip` (hidden, internal).
    #[command(name = "__clip-clear", hide = true)]
    ClipClear(ClipClearArgs),

    /// Panic on purpose, to exercise the panic hook (hidden, debug only).
    #[cfg(debug_assertions)]
    #[command(name = "__panic-test", hide = true)]
    PanicTest {
        /// Ignored. Exists so a test can prove that no argument value
        /// reaches the panic report or stderr.
        #[arg(trailing_var_arg = true, hide = true)]
        args: Vec<String>,
    },
}

/// Flags shared by `init` and `migrate` for choosing the target store's
/// kind and recipients (3.5.2, 3.5.15).
#[derive(Debug, Clone, Args)]
pub struct TargetArgs {
    /// A recipient (public key) to encrypt to: `age1...`, an SSH public
    /// key, or a plugin recipient. Repeatable.
    #[arg(long = "recipient", value_name = "R")]
    pub recipient: Vec<String>,

    /// A file of recipients, one per line (blank lines and `#` comments
    /// ignored).
    #[arg(long = "recipients-file", value_name = "PATH")]
    pub recipients_file: Option<PathBuf>,

    /// Create a passphrase store instead of a recipients store.
    #[arg(long, conflicts_with_all = ["recipient", "recipients_file", "no_self"])]
    pub passphrase: bool,

    /// Do not add the caller's own recipient to a recipients store.
    #[arg(long = "no-self")]
    pub no_self: bool,
}

/// `trousseau init` (3.5.2).
#[derive(Debug, Args)]
pub struct InitArgs {
    /// The target store's kind and recipients.
    #[command(flatten)]
    pub target: TargetArgs,
}

/// `trousseau set` (3.5.4).
#[derive(Debug, Args)]
pub struct SetArgs {
    /// The key to set: a path such as `database/password`. Segments use
    /// letters, digits, `.`, `_` and `-`, and are separated by `/`. Keys
    /// are case-sensitive.
    pub key: String,

    /// Catches `set KEY VALUE` so the command can explain itself instead
    /// of failing with a generic usage error. Hidden from help.
    #[arg(hide = true, value_name = "VALUE", value_parser = discard_value, num_args = 0..)]
    pub rejected_value: Vec<RejectedValue>,

    /// Read the value from this file. `-` means stdin, verbatim.
    #[arg(long = "from-file", value_name = "PATH", conflicts_with = "from_env")]
    pub from_file: Option<PathBuf>,

    /// Read the value from this environment variable.
    #[arg(long = "from-env", value_name = "NAME")]
    pub from_env: Option<String>,

    /// Store the value as binary (base64) even if it is valid text.
    /// Binary entries are skipped by `run` and `env`.
    #[arg(long)]
    pub binary: bool,

    /// The variable name `run` and `env` use for this entry, instead of
    /// the name derived from the key.
    #[arg(long, value_name = "NAME")]
    pub env: Option<String>,

    /// A free-text description of the entry.
    #[arg(long, value_name = "TEXT")]
    pub description: Option<String>,
}

/// Marker for a positional `VALUE` given to `set`. The text itself is
/// discarded at parse time: `set` never accepts a value on the command
/// line, and this keeps the secret out of any `Debug` output.
#[derive(Debug, Clone, Copy)]
pub struct RejectedValue;

/// Parse any string into [`RejectedValue`], dropping the input.
#[allow(clippy::unnecessary_wraps)]
const fn discard_value(_: &str) -> Result<RejectedValue, std::convert::Infallible> {
    Ok(RejectedValue)
}

/// `trousseau get` (3.5.5).
#[derive(Debug, Args)]
pub struct GetArgs {
    /// The key to read.
    pub key: String,

    /// Write the value to this file instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

    /// Overwrite `--out` if it already exists.
    #[arg(long)]
    pub force: bool,

    /// Copy the value to the clipboard instead of printing it.
    #[arg(long)]
    pub clip: bool,
}

/// `trousseau ls` (3.5.6).
#[derive(Debug, Args)]
pub struct LsArgs {
    /// Only list keys equal to, or nested under, this path prefix.
    pub prefix: Option<String>,

    /// Print a table with encoding, env name, update time, and
    /// description columns. Never values.
    #[arg(long)]
    pub long: bool,
}

/// `trousseau rm` (3.5.7).
#[derive(Debug, Args)]
pub struct RmArgs {
    /// The keys to remove.
    #[arg(required = true)]
    pub keys: Vec<String>,

    /// Ignore keys that do not exist instead of failing.
    #[arg(long)]
    pub force: bool,
}

/// `trousseau mv` (3.5.8).
#[derive(Debug, Args)]
pub struct MvArgs {
    /// The existing key.
    pub old: String,

    /// The new key.
    pub new: String,

    /// Overwrite `new` if it already exists.
    #[arg(long)]
    pub force: bool,
}

/// `trousseau recipients <ls|add|rm>` (3.5.9).
#[derive(Debug, Subcommand)]
pub enum RecipientsAction {
    /// List the store's recipients, one per line.
    Ls,
    /// Add one or more recipients and re-encrypt the store to the new
    /// list.
    ///
    /// Recipients already in the list are skipped with a note.
    #[command(after_help = crate::help::RECIPIENTS_ADD)]
    Add {
        /// The recipients to add.
        #[arg(required = true)]
        recipients: Vec<String>,
    },
    /// Remove one or more recipients and re-encrypt the store.
    ///
    /// SSH keys match on the key itself, so the trailing comment does
    /// not have to be the same. The last recipient cannot be removed.
    /// Removing your own recipient asks for confirmation, because you
    /// lock yourself out.
    #[command(after_help = crate::help::RECIPIENTS_RM)]
    Rm {
        /// The recipients to remove.
        #[arg(required = true)]
        recipients: Vec<String>,
        /// Skip the "you removed your own recipient" confirmation.
        #[arg(long)]
        force: bool,
    },
}

/// `trousseau rekey` (3.5.10).
#[derive(Debug, Args)]
pub struct RekeyArgs {
    /// Convert to a passphrase store, or change the passphrase. Prompts
    /// twice.
    #[arg(long = "to-passphrase", conflicts_with = "to_recipients")]
    pub to_passphrase: bool,

    /// Convert to a recipients store with exactly these recipients.
    #[arg(long = "to-recipients", value_name = "R", num_args = 1..)]
    pub to_recipients: Option<Vec<String>>,
}

/// The export/import payload format (3.5.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum PayloadFormat {
    /// The full JSON payload document.
    Json,
    /// One `NAME="value"` line per UTF-8 entry.
    Dotenv,
    /// The `edit` TOML document format.
    Toml,
}

/// `trousseau export` (3.5.11).
#[derive(Debug, Args)]
pub struct ExportArgs {
    /// The output format.
    #[arg(long, value_enum, default_value_t = PayloadFormat::Json)]
    pub format: PayloadFormat,

    /// Write to this file instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

    /// Overwrite `--out` if it already exists.
    #[arg(long)]
    pub force: bool,
}

/// How `import` resolves a key that already exists (3.5.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum ImportStrategy {
    /// Keep the existing entry.
    Keep,
    /// Replace the existing entry with the imported one.
    Overwrite,
    /// Abort the whole import on any collision.
    Fail,
}

/// `trousseau import` (3.5.11).
#[derive(Debug, Args)]
pub struct ImportArgs {
    /// The input format.
    #[arg(long, value_enum, default_value_t = PayloadFormat::Json)]
    pub format: PayloadFormat,

    /// How to resolve a key that already exists.
    #[arg(long, value_enum, default_value_t = ImportStrategy::Fail)]
    pub strategy: ImportStrategy,

    /// Read from this file instead of stdin.
    pub path: Option<PathBuf>,
}

/// `trousseau run` (3.5.13).
#[derive(Debug, Args)]
pub struct RunArgs {
    /// A prefix added in front of every variable name. Defaults to
    /// `[run].env_prefix` in the config file.
    #[arg(long = "env-prefix", value_name = "P")]
    pub env_prefix: Option<String>,

    /// Only inject this key, or the keys under this path prefix.
    /// Repeatable.
    #[arg(long = "only", value_name = "KEYPREFIX")]
    pub only: Vec<String>,

    /// Give the child only the injected variables plus a small allow
    /// list, instead of inheriting the parent's environment.
    #[arg(long = "no-inherit")]
    pub no_inherit: bool,

    /// The command to run, and its arguments. Put it after `--`.
    #[arg(last = true, required = true, value_name = "CMD")]
    pub cmd: Vec<String>,
}

/// The output format for `trousseau env` (3.5.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "lowercase")]
pub enum EnvFormat {
    /// `export NAME='value'` lines.
    #[default]
    Shell,
    /// `NAME="value"` lines.
    Dotenv,
    /// A single `{"NAME": "value"}` JSON object.
    Json,
}

/// `trousseau env` (3.5.14).
#[derive(Debug, Args)]
pub struct EnvArgs {
    /// The output format. `--json` is an alias for `--format json`.
    #[arg(long, value_enum, default_value_t = EnvFormat::Shell)]
    pub format: EnvFormat,

    /// A prefix added in front of every variable name. Defaults to
    /// `[run].env_prefix` in the config file.
    #[arg(long = "env-prefix", value_name = "P")]
    pub env_prefix: Option<String>,

    /// Only print this key, or the keys under this path prefix.
    /// Repeatable.
    #[arg(long = "only", value_name = "KEYPREFIX")]
    pub only: Vec<String>,
}

/// `trousseau migrate` (3.5.15).
#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// The legacy v0.4 store file to read.
    pub source: PathBuf,

    /// The `gpg` binary to use for an `OpenPGP` legacy store.
    #[arg(long, value_name = "PATH")]
    pub gpg: Option<PathBuf>,

    /// An isolated `GNUPGHOME` to use when invoking `gpg`.
    #[arg(long = "gnupg-home", value_name = "PATH")]
    pub gnupg_home: Option<PathBuf>,

    /// A file holding the new store's passphrase (only with
    /// `--passphrase`).
    #[arg(long = "new-passphrase-file", value_name = "PATH")]
    pub new_passphrase_file: Option<PathBuf>,

    /// The target store's kind and recipients.
    #[command(flatten)]
    pub target: TargetArgs,
}

/// `trousseau completions` (3.5.17).
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// The shell to generate a completion script for.
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

/// `trousseau man` (3.5.17).
#[derive(Debug, Args)]
pub struct ManArgs {
    /// Print the man page for this subcommand instead of the top level.
    pub subcommand: Option<String>,
}

/// `trousseau __clip-clear` (3.5.16, hidden).
#[derive(Debug, Args)]
pub struct ClipClearArgs {
    /// The lowercase hex SHA-256 digest of the value that was copied.
    pub hash: String,

    /// How many seconds to wait before clearing the clipboard.
    pub seconds: u64,
}
