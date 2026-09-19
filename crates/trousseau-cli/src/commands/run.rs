//! `trousseau run` (3.5.13). Implemented in step 3.6.

use crate::cli::RunArgs;
use crate::context::Context;

/// Run `run`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.6.
pub fn run(_ctx: &Context, _args: &RunArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.6)"))
}
