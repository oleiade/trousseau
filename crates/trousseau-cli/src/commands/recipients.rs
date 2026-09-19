//! `trousseau recipients ls|add|rm` (3.5.9). Implemented in step 3.4.

use crate::context::Context;

/// Run `recipients ls`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.4.
pub fn run_ls(_ctx: &Context) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.4)"))
}

/// Run `recipients add`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.4.
pub fn run_add(_ctx: &Context, _recipients: &[String]) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.4)"))
}

/// Run `recipients rm`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.4.
pub fn run_rm(_ctx: &Context, _recipients: &[String], _force: bool) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.4)"))
}
