//! `trousseau __clip-clear` (3.5.16, hidden). Implemented in step 3.9.

use crate::cli::ClipClearArgs;
use crate::context::Context;

/// Run `__clip-clear`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.9.
pub fn run(_ctx: &Context, _args: &ClipClearArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.9)"))
}
