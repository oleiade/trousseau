//! `trousseau completions` (3.5.17).

use clap::CommandFactory as _;

use crate::cli::{Cli, CompletionsArgs};

/// Run `completions`: print `args.shell`'s completion script, generated
/// by `clap_complete` from the same [`Cli`] tree `cli.rs` declares, to
/// stdout.
///
/// The `Result` return type is dictated by [`crate::commands::dispatch`],
/// which every subcommand handler must match; `clap_complete::generate`
/// itself cannot fail (a stdout write error is not surfaced by its
/// `Write`-based API).
///
/// # Errors
///
/// Never returns an error.
#[allow(clippy::unnecessary_wraps)]
pub fn run(args: &CompletionsArgs) -> anyhow::Result<()> {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();
    let mut stdout = crate::output::writer();
    clap_complete::generate(args.shell, &mut command, name, &mut stdout);
    Ok(())
}
