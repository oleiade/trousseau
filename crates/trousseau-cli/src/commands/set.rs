//! `trousseau set` (3.5.4). Implemented in step 3.3.

use crate::cli::SetArgs;
use crate::context::Context;

/// Run `set`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.3.
pub fn run(_ctx: &Context, _args: &SetArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.3)"))
}
