//! `trousseau completions` (3.5.17). Implemented in step 3.9.

use crate::cli::CompletionsArgs;

/// Run `completions`.
///
/// # Errors
///
/// Always returns a "not implemented yet" error until step 3.9.
pub fn run(_args: &CompletionsArgs) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("not implemented yet (step 3.9)"))
}
