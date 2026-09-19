//! `trousseau export` (3.5.11). Implemented in step 3.5.

use crate::cli::ExportArgs;
use crate::context::Context;

/// Run `export`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.5.
pub fn run(_ctx: &Context, _args: &ExportArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.5)"))
}
