//! `trousseau env` (3.5.14). Implemented in step 3.6.

use crate::cli::EnvArgs;
use crate::context::Context;

/// Run `env`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.6.
pub fn run(_ctx: &Context, _args: &EnvArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.6)"))
}
