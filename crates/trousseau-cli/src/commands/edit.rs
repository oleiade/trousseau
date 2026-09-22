//! `trousseau edit` (3.5.12).
//!
//! The exclusive store lock is acquired before the scratch file is
//! written or the editor is spawned, and held until the store is saved
//! (or the command gives up without saving): see [`run`]. The scratch
//! file itself is a [`tempfile::NamedTempFile`], whose own `Drop`
//! implementation is the "delete on every path, including panics" guard
//! 3.5.12 calls for; nothing here needs a second one.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Context as _;
use serde::Serialize;
use time::OffsetDateTime;

use trousseau::schema::{Entry, Key, Store};
use trousseau::store::LockMode;

use crate::context::Context;
use crate::document::{self, DocEntry};
use crate::output::{self, OutputMode};
use crate::prompt;

/// Prefix for the scratch file's name (3.5.12).
const SCRATCH_PREFIX: &str = "trousseau-";
/// Suffix for the scratch file's name (3.5.12).
const SCRATCH_SUFFIX: &str = ".toml";

/// Run `edit`.
///
/// # Errors
///
/// Returns whatever [`Context::unlock`] returns, an error if the scratch
/// file cannot be created or the editor cannot be spawned, an error if
/// the editor exits with a non-zero status, an error if the edited
/// document is still not valid TOML after every reopen (or immediately,
/// with `--no-input` or a non-terminal stdin), or whatever
/// [`Context::seal_for`] or [`trousseau::store::save`] returns.
pub fn run(ctx: &Context) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store();
    let path = resolved.path();

    // Held from before the scratch file (and the editor operating on
    // it) exists until the save below completes (3.5.12).
    let _lock = ctx.lock(path, LockMode::Exclusive)?;
    let mut store = ctx.unlock(path)?;

    let header = format!(
        "# trousseau edit: {}\n# Save and quit to apply. Delete a table to remove its entry.\n# Leave the file empty to abort.",
        path.display()
    );
    let original = document::to_toml(&store, &header);

    let scratch = create_scratch()?;
    write_scratch(&scratch, &original)?;
    // Close our own handle before the editor opens the file: Windows
    // refuses to let another process truncate or replace a file we still
    // hold open. The `TempPath` keeps the delete-on-drop guarantee.
    let scratch = scratch.into_temp_path();
    let scratch_path = scratch.to_path_buf();

    let parsed = loop {
        run_editor(&scratch_path)?;
        let current = std::fs::read_to_string(&scratch_path)
            .with_context(|| format!("reading {}", scratch_path.display()))?;

        if current.trim().is_empty() || current == original {
            // `scratch` (a `TempPath`) is dropped, and so deleted,
            // when this function returns.
            return report_no_changes(ctx);
        }

        match document::from_toml(&current) {
            Ok(parsed) => break parsed,
            Err(err) => {
                output::warn(&format!("error: {err}"));
                if ctx.no_input || !ctx.is_stdin_tty {
                    anyhow::bail!("invalid toml document; edit aborted");
                }
                if prompt::confirm("Reopen the editor? [Y/n]", true)? {
                    continue;
                }
                anyhow::bail!("invalid toml document; edit aborted");
            }
        }
    };

    let changed = apply(&mut store, parsed, ctx.now());

    let seal_material = ctx.seal_for(&store)?;
    trousseau::store::save(path, &store, seal_material.as_seal())?;

    // `scratch` is dropped (and deleted) here, after the save completes.
    drop(scratch);

    report_saved(ctx, changed)
}

/// Create the scratch file `tempfile::Builder` with prefix `trousseau-`
/// and suffix `.toml`, mode `0600` on Unix, in [`scratch_dir`] (3.5.12).
fn create_scratch() -> anyhow::Result<tempfile::NamedTempFile> {
    let dir = scratch_dir();
    let mut builder = tempfile::Builder::new();
    builder.prefix(SCRATCH_PREFIX).suffix(SCRATCH_SUFFIX);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        builder.permissions(std::fs::Permissions::from_mode(0o600));
    }
    builder
        .tempfile_in(&dir)
        .with_context(|| format!("creating a scratch file in {}", dir.display()))
}

/// Write `text` to `scratch`.
fn write_scratch(scratch: &tempfile::NamedTempFile, text: &str) -> anyhow::Result<()> {
    scratch
        .as_file()
        .write_all(text.as_bytes())
        .with_context(|| format!("writing scratch file {}", scratch.path().display()))
}

/// The scratch directory (3.5.12): the first existing directory of
/// `$XDG_RUNTIME_DIR`, `/dev/shm`, and the system temp directory, on
/// Unix. On Windows, always the system temp directory.
#[cfg(unix)]
fn scratch_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from)
        && dir.is_dir()
    {
        return dir;
    }
    let shm = PathBuf::from("/dev/shm");
    if shm.is_dir() {
        return shm;
    }
    std::env::temp_dir()
}

/// See the Unix [`scratch_dir`]: on every other platform, always the
/// system temp directory (3.5.12).
#[cfg(not(unix))]
fn scratch_dir() -> PathBuf {
    std::env::temp_dir()
}

/// Spawn the editor on `scratch_path`, waiting for it to exit.
///
/// `command_line` is a whole shell command (`$VISUAL`/`$EDITOR` can be
/// `"code --wait"`, not just a bare program name), so rather than
/// splitting it by hand, it is handed to the platform's own shell: `sh
/// -c` on Unix, with `scratch_path` passed as `$1` so it never has to be
/// escaped into the command string, and `cmd /C` on Windows, where
/// simple quoting is safe because `"` is a reserved character no valid
/// path can contain.
///
/// # Errors
///
/// Returns an error if the shell cannot be spawned, or if the editor
/// exits with a non-zero status (3.5.12: abort, exit 1, scratch
/// deleted).
fn run_editor(scratch_path: &Path) -> anyhow::Result<()> {
    let command_line = editor_command();
    let status = spawn_editor(&command_line, scratch_path)
        .with_context(|| format!("running editor {command_line:?}"))?;

    if !status.success() {
        anyhow::bail!("editor exited with a non-zero status");
    }
    Ok(())
}

#[cfg(unix)]
fn spawn_editor(
    command_line: &str,
    scratch_path: &Path,
) -> std::io::Result<std::process::ExitStatus> {
    Command::new("sh")
        .arg("-c")
        .arg(format!("{command_line} \"$1\""))
        .arg("sh") // $0: conventionally the program name, unused here
        .arg(scratch_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

#[cfg(not(unix))]
fn spawn_editor(
    command_line: &str,
    scratch_path: &Path,
) -> std::io::Result<std::process::ExitStatus> {
    Command::new("cmd")
        .arg("/C")
        .arg(format!("{command_line} \"{}\"", scratch_path.display()))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

/// The editor command line: `$VISUAL`, else `$EDITOR`, else `vi` on Unix
/// or `notepad` on Windows (3.5.12). A set-but-blank `$VISUAL`/`$EDITOR`
/// is treated the same as unset.
fn editor_command() -> String {
    for var in ["VISUAL", "EDITOR"] {
        if let Ok(value) = std::env::var(var)
            && !value.trim().is_empty()
        {
            return value;
        }
    }
    default_editor().to_owned()
}

/// The fallback editor on Unix (3.5.12).
#[cfg(unix)]
const fn default_editor() -> &'static str {
    "vi"
}

/// The fallback editor on Windows (3.5.12).
#[cfg(windows)]
const fn default_editor() -> &'static str {
    "notepad"
}

/// Apply the parsed document to `store`, per 3.5.12: a table missing
/// from `parsed` that used to exist removes that entry; a table whose
/// value or metadata differs from the existing entry bumps its
/// `updated_at` (keeping `created_at`); a table identical to the
/// existing entry leaves it untouched, timestamps included; a table with
/// no existing entry creates one with both timestamps set to `now`.
///
/// Returns `true` if the store actually changed (so the store-level
/// `updated_at` was bumped too).
fn apply(store: &mut Store, parsed: BTreeMap<Key, DocEntry>, now: OffsetDateTime) -> bool {
    let now = trousseau::schema::truncate_to_seconds(now);
    let mut changed = false;

    let existing_keys: Vec<Key> = store.entries.keys().cloned().collect();
    for key in existing_keys {
        if !parsed.contains_key(&key) {
            store.entries.remove(&key);
            changed = true;
        }
    }

    for (key, doc_entry) in parsed {
        match store.entries.get(&key) {
            Some(existing) if entry_matches(existing, &doc_entry) => {
                // Unchanged: keep the existing timestamps untouched.
            }
            Some(existing) => {
                let created_at = existing.created_at;
                store.entries.insert(
                    key,
                    Entry {
                        value: doc_entry.value,
                        encoding: doc_entry.encoding,
                        env: doc_entry.env,
                        description: doc_entry.description,
                        created_at,
                        updated_at: now,
                    },
                );
                changed = true;
            }
            None => {
                store.entries.insert(
                    key,
                    Entry {
                        value: doc_entry.value,
                        encoding: doc_entry.encoding,
                        env: doc_entry.env,
                        description: doc_entry.description,
                        created_at: now,
                        updated_at: now,
                    },
                );
                changed = true;
            }
        }
    }

    if changed {
        store.updated_at = now;
    }
    changed
}

/// `true` if `existing`'s value, encoding, `env`, and `description` are
/// exactly what `doc_entry` carries (3.5.12: this decides whether a
/// table counts as "unchanged").
fn entry_matches(existing: &Entry, doc_entry: &DocEntry) -> bool {
    existing.value == doc_entry.value
        && existing.encoding == doc_entry.encoding
        && existing.env == doc_entry.env
        && existing.description == doc_entry.description
}

/// The `edit` output shape for `--json` (3.5.1's general "write commands
/// emit `{\"ok\": true, ...}`" rule; `edit` has no shape of its own in
/// appendix 5.1).
#[derive(Serialize)]
struct EditJson {
    ok: bool,
    changed: bool,
}

/// Report the "no changes" outcome (3.5.12): `no changes` on stderr in
/// human mode, or `{"ok": true, "changed": false}` on stdout in `--json`
/// mode.
fn report_no_changes(ctx: &Context) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            output::info(ctx.quiet, "no changes");
            Ok(())
        }
        OutputMode::Json => output::json(&EditJson {
            ok: true,
            changed: false,
        }),
    }
}

/// Report a successful save (3.5.1's general write-command contract;
/// `edit` has no human-mode message of its own specified beyond `no
/// changes`, so this mirrors `set`/`mv`/`rm`'s "one short past-tense
/// line" convention).
fn report_saved(ctx: &Context, changed: bool) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            output::info(ctx.quiet, "edited");
            Ok(())
        }
        OutputMode::Json => output::json(&EditJson { ok: true, changed }),
    }
}
