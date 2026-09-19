//! Hidden input, confirmations, and the `age::Callbacks` implementor.
//!
//! Every function here that prompts respects `--no-input`
//! ([`Context::no_input`](crate::context::Context::no_input)) at its call
//! site in `context.rs`; the low-level functions in this module always
//! attempt to read from the terminal when called, since some callers
//! (`age`'s own plugin protocol, through [`CliCallbacks`]) need to make
//! that decision per call rather than once up front.
//!
//! Not yet called from any command in step 3.1 (see `context.rs`'s note
//! on the same `#![allow(dead_code)]`); `context.rs` starts using it in
//! step 3.2.
#![allow(dead_code)]

use secrecy::SecretString;

use crate::output;

/// The minimum length, in bytes, of a passphrase for a brand-new
/// passphrase store (3.3.3).
const MIN_PASSPHRASE_BYTES: usize = 8;

/// Prompt `prompt` and read a line of hidden (not echoed) input.
///
/// # Errors
///
/// Returns an error if the terminal cannot be read.
pub fn hidden(prompt: &str) -> anyhow::Result<SecretString> {
    let value = rpassword::prompt_password(prompt)?;
    Ok(SecretString::from(value))
}

/// Prompt twice for a brand-new passphrase, comparing the two entries
/// and enforcing the 8-byte minimum (3.3.3), reprompting on a mismatch
/// or a too-short entry.
///
/// # Errors
///
/// Returns an error if the terminal cannot be read.
pub fn hidden_confirm(prompt: &str) -> anyhow::Result<SecretString> {
    use secrecy::ExposeSecret as _;
    loop {
        let first = hidden(prompt)?;
        let second = hidden("Confirm passphrase: ")?;
        if first.expose_secret() != second.expose_secret() {
            output::warn("passphrases did not match; try again");
            continue;
        }
        if first.expose_secret().len() < MIN_PASSPHRASE_BYTES {
            output::warn("passphrase must be at least 8 bytes; try again");
            continue;
        }
        return Ok(first);
    }
}

/// Ask a yes/no question, defaulting to `default` on a bare Enter.
///
/// # Errors
///
/// Returns an error if the terminal cannot be read.
pub fn confirm(question: &str, default: bool) -> anyhow::Result<bool> {
    let answer = dialoguer::Confirm::new()
        .with_prompt(question)
        .default(default)
        .interact()?;
    Ok(answer)
}

/// The `age::Callbacks` implementor used everywhere this crate hands
/// identities or recipients to the `age` crate (plugin prompts,
/// encrypted SSH and age identity passphrases).
///
/// `age::Callbacks` requires `Clone + Send + Sync + 'static`;
/// `CliCallbacks` holds only the flag it needs (`no_input`), so cloning
/// it is cheap and it never carries a reference back to a
/// [`Context`](crate::context::Context).
#[derive(Debug, Clone, Copy)]
pub struct CliCallbacks {
    /// When `true`, every callback that could prompt returns `None`
    /// instead (3.5.1: `--no-input` never prompts).
    no_input: bool,
}

impl CliCallbacks {
    /// Build a new callbacks implementor. `no_input` mirrors
    /// [`Context::no_input`](crate::context::Context::no_input).
    #[must_use]
    pub const fn new(no_input: bool) -> Self {
        Self { no_input }
    }
}

impl age::Callbacks for CliCallbacks {
    fn display_message(&self, message: &str) {
        output::warn(message);
    }

    fn confirm(&self, message: &str, yes_string: &str, no_string: Option<&str>) -> Option<bool> {
        if self.no_input {
            return None;
        }
        let question = no_string.map_or_else(
            || format!("{message} [{yes_string}]"),
            |no| format!("{message} [{yes_string}/{no}]"),
        );
        confirm(&question, false).ok()
    }

    fn request_public_string(&self, description: &str) -> Option<String> {
        if self.no_input {
            return None;
        }
        output::warn(description);
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).ok()?;
        Some(line.trim_end_matches(['\n', '\r']).to_owned())
    }

    fn request_passphrase(&self, description: &str) -> Option<SecretString> {
        if self.no_input {
            return None;
        }
        hidden(&format!("{description}: ")).ok()
    }
}
