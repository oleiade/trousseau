//! `trousseau rm` (3.5.7). Implemented in step 3.3.

use crate::cli::RmArgs;
use crate::context::Context;

/// Run `rm`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.3.
pub fn run(_ctx: &Context, _args: &RmArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.3)"))
}
