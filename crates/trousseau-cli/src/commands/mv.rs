//! `trousseau mv` (3.5.8). Implemented in step 3.3.

use crate::cli::MvArgs;
use crate::context::Context;

/// Run `mv`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.3.
pub fn run(_ctx: &Context, _args: &MvArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.3)"))
}
