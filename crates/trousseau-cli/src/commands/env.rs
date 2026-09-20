//! `trousseau env` (3.5.14).

use std::collections::BTreeMap;

use trousseau::schema::Entry;
use trousseau::store::LockMode;

use crate::cli::{EnvArgs, EnvFormat};
use crate::context::Context;
use crate::document;
use crate::output::{self, OutputMode};

/// Run `env`.
///
/// # Errors
///
/// Returns whatever [`Context::unlock`] returns,
/// [`trousseau::error::Error::InvalidStore`] if `--env-prefix` does not
/// match the environment-name grammar (3.1.4), or
/// [`trousseau::error::Error::EnvConflict`] if two selected entries
/// resolve to the same environment variable name.
pub fn run(ctx: &Context, args: &EnvArgs) -> anyhow::Result<()> {
    let prefix = args.env_prefix.as_deref().unwrap_or("");

    let resolved = ctx.resolve_store();
    let path = resolved.path();
    let _lock = ctx.lock(path, LockMode::Shared)?;
    let store = ctx.unlock(path)?;

    let env_map = super::resolve_env_selection(&store, prefix, &args.only)?;

    // Global `--json` is an alias for `--format json` (3.5.14).
    let format = if ctx.output == OutputMode::Json {
        EnvFormat::Json
    } else {
        args.format
    };

    match format {
        EnvFormat::Shell => print_shell(&env_map),
        EnvFormat::Dotenv => print_dotenv(&env_map),
        EnvFormat::Json => print_json(&env_map),
    }
}

/// An entry's value as a `String`. `env_map` only ever contains `utf8`
/// entries (`base64` entries are excluded, 3.1.4), whose bytes are
/// always valid UTF-8 (checked at every write path); `unwrap_or_default`
/// is a defensive fallback, never expected to trigger.
fn entry_value(entry: &Entry) -> String {
    std::str::from_utf8(entry.value.expose())
        .unwrap_or_default()
        .to_owned()
}

/// `export NAME='value'` lines, one per selected entry, in name order
/// (`env_map`'s `BTreeMap`); `'` in `value` becomes `'\''` (3.5.14),
/// which is the only character that needs escaping inside a POSIX
/// single-quoted string, including a literal newline.
fn print_shell(map: &BTreeMap<String, &Entry>) -> anyhow::Result<()> {
    let mut out = String::new();
    for (name, entry) in map {
        out.push_str("export ");
        out.push_str(name);
        out.push_str("='");
        out.push_str(&escape_shell_value(&entry_value(entry)));
        out.push_str("'\n");
    }
    output::raw(out.as_bytes())
}

/// Escape a value for a POSIX single-quoted shell string: `'` becomes
/// `'\''` (close the quote, an escaped literal `'`, reopen the quote)
/// (3.5.14).
fn escape_shell_value(text: &str) -> String {
    text.replace('\'', "'\\''")
}

/// `NAME="value"` lines, one per selected entry, in name order, using
/// the same escaping as `export --format dotenv` (3.5.11, 3.5.14).
fn print_dotenv(map: &BTreeMap<String, &Entry>) -> anyhow::Result<()> {
    let mut out = String::new();
    for (name, entry) in map {
        out.push_str(&document::dotenv_line(name, &entry_value(entry)));
    }
    output::raw(out.as_bytes())
}

/// `{"NAME": "value"}` (appendix 5.1).
fn print_json(map: &BTreeMap<String, &Entry>) -> anyhow::Result<()> {
    let object: BTreeMap<&str, String> = map
        .iter()
        .map(|(name, entry)| (name.as_str(), entry_value(entry)))
        .collect();
    output::json(&object)
}
