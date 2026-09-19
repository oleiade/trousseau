//! One module per subcommand, dispatched from [`dispatch`].
//!
//! Every command not yet implemented (everything before its plan step
//! lands) returns `anyhow::anyhow!("not implemented yet (step N)")` with
//! the step from `docs/IMPLEMENTATION_PLAN.md` section 4 that implements
//! it. `main.rs` maps that generic error to exit code 1 through
//! `exit.rs`, same as any other unexpected failure.

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
