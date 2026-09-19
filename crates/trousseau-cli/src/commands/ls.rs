//! `trousseau ls` (3.5.6). Implemented in step 3.3.

use crate::cli::LsArgs;
use crate::context::Context;

/// Run `ls`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.3.
pub fn run(_ctx: &Context, _args: &LsArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.3)"))
}
