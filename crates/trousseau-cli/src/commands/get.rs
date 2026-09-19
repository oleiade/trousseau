//! `trousseau get` (3.5.5). Implemented in step 3.3 (`--clip` in 3.9).

use crate::cli::GetArgs;
use crate::context::Context;

/// Run `get`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.3.
pub fn run(_ctx: &Context, _args: &GetArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.3)"))
}
