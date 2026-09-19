//! `trousseau man` (3.5.17). Implemented in step 3.9.

use crate::cli::ManArgs;

/// Run `man`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.9.
pub fn run(_args: &ManArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.9)"))
}
