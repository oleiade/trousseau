//! `trousseau info` (3.5.3). Implemented in step 3.2.

use crate::context::Context;

/// Run `info`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.2.
pub fn run(_ctx: &Context) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.2)"))
}
