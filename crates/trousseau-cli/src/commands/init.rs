//! `trousseau init` (3.5.2). Implemented in step 3.2.

use crate::cli::InitArgs;
use crate::context::Context;

/// Run `init`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.2.
pub fn run(_ctx: &Context, _args: &InitArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.2)"))
}
