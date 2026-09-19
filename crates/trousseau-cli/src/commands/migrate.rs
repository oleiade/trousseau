//! `trousseau migrate` (3.5.15). Implemented in step 3.8.

use crate::cli::MigrateArgs;
use crate::context::Context;

/// Run `migrate`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.8.
pub fn run(_ctx: &Context, _args: &MigrateArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.8)"))
}
