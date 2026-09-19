//! `trousseau rekey` (3.5.10). Implemented in step 3.4.

use crate::cli::RekeyArgs;
use crate::context::Context;

/// Run `rekey`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.4.
pub fn run(_ctx: &Context, _args: &RekeyArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.4)"))
}
