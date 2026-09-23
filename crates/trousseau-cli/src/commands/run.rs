//! `trousseau run` (3.5.13).

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::Context as _;

use trousseau::schema::Entry;
use trousseau::store::LockMode;

use crate::cli::RunArgs;
use crate::context::Context;
use crate::exit::CliError;
use crate::output::OutputMode;

/// Environment variable names `--no-inherit` keeps from the parent
/// process's environment verbatim, in addition to any `LC_*` variable
/// that is set (3.5.13).
const NO_INHERIT_ALLOWLIST: [&str; 5] = ["PATH", "HOME", "TMPDIR", "TERM", "LANG"];

/// Run `run`.
///
/// # Errors
///
/// Returns [`CliError::Usage`] if global `--json` is set (`run` has no
/// JSON output, 3.5.13), whatever [`Context::unlock`] returns,
/// [`trousseau::error::Error::EnvConflict`] if two selected entries
/// resolve to the same environment variable name (checked, and
/// reported, before `CMD` is executed), or an I/O error spawning or
/// executing `CMD`.
pub fn run(ctx: &Context, args: &RunArgs) -> anyhow::Result<()> {
    if ctx.output == OutputMode::Json {
        return Err(CliError::Usage("--json is not supported by run".to_owned()).into());
    }

    let prefix = args
        .env_prefix
        .as_deref()
        .unwrap_or(&ctx.config.run.env_prefix);

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let lock = ctx.lock(path, LockMode::Shared)?;
    let store = ctx.unlock(path)?;

    let env_map = super::resolve_env_selection(&store, prefix, &args.only)?;
    let mut command = build_command(&args.cmd, &env_map, args.no_inherit)?;

    // Nothing is written to disk by `run`, and the child must not hold
    // the store locked for however long it runs: release the shared
    // lock now, before the child is spawned (Unix: before the process
    // image is replaced) (3.5.13).
    drop(lock);

    exec_child(&mut command)
}

/// Build the child [`Command`]: `cmd[0]` as the program, `cmd[1..]` as
/// its arguments, the parent's environment (or, under `--no-inherit`,
/// only [`NO_INHERIT_ALLOWLIST`] and any set `LC_*` variable), then
/// `env_map`'s entries layered on top so they always win a name
/// collision (3.5.13).
fn build_command(
    cmd: &[String],
    env_map: &BTreeMap<String, &Entry>,
    no_inherit: bool,
) -> anyhow::Result<Command> {
    let (program, rest) = cmd
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("run: no command given"))?;
    let mut command = Command::new(program);
    command.args(rest);

    if no_inherit {
        command.env_clear();
        for name in NO_INHERIT_ALLOWLIST {
            if let Ok(value) = std::env::var(name) {
                command.env(name, value);
            }
        }
        for (name, value) in std::env::vars() {
            if name.starts_with("LC_") {
                command.env(name, value);
            }
        }
    }

    for (name, entry) in env_map {
        command.env(name, super::entry_text(entry));
    }

    Ok(command)
}

/// Unix: replace the current process with `CMD` (3.5.13). Only returns
/// on failure: `exec` never returns on success.
#[cfg(unix)]
fn exec_child(command: &mut Command) -> anyhow::Result<()> {
    use std::os::unix::process::CommandExt as _;
    let err = command.exec();
    Err(err).context("executing child command")
}

/// Windows: spawn `CMD`, wait for it, then exit this process with its
/// exit code (3.5.13).
///
/// # Errors
///
/// Returns [`CliError::ChildFailed`] if the child's exit status cannot
/// be represented as an exit code (Windows only), or an I/O error if the
/// child could not be spawned or waited on. Never returns `Ok`: the
/// child's own exit code ends the process directly.
#[cfg(windows)]
fn exec_child(command: &mut Command) -> anyhow::Result<()> {
    let status = command.status().context("spawning child command")?;
    match status.code() {
        Some(code) => std::process::exit(code),
        None => Err(CliError::ChildFailed.into()),
    }
}
