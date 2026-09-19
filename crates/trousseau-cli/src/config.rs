//! `<config_dir>/trousseau/config.toml` (3.4).
//!
//! The configuration file is optional. A missing file yields
//! [`Config::default`]; a malformed one (unknown keys included) is an
//! error whose message names the offending file and key.
//!
//! `timeout_seconds`, `env_prefix`, and `gpg` are not yet read by any
//! command as of step 3.2; `get --clip`, `run`/`env`, and `migrate`
//! start reading them in later steps.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::Deserialize;

/// The parsed configuration file (3.4).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// `[identity]`: additional identity files.
    #[serde(default)]
    pub identity: IdentityConfig,

    /// `[clipboard]`: clipboard-clearing behavior.
    #[serde(default)]
    pub clipboard: ClipboardConfig,

    /// `[run]`: defaults for `run` and `env`.
    #[serde(default)]
    pub run: RunConfig,

    /// `[migrate]`: defaults for `migrate`.
    #[serde(default)]
    pub migrate: MigrateConfig,
}

/// `[identity]` (3.4).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityConfig {
    /// Additional identity files, tried in order after `--identity` and
    /// `TROUSSEAU_IDENTITY_FILE` (3.3.2). `~` at the start of an entry
    /// expands to the home directory.
    #[serde(default)]
    pub files: Vec<String>,
}

/// `[clipboard]` (3.4).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipboardConfig {
    /// Seconds before the clipboard is cleared after `get --clip`. `0`
    /// disables clearing.
    #[serde(default = "default_clipboard_timeout_seconds")]
    pub timeout_seconds: u64,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_clipboard_timeout_seconds(),
        }
    }
}

/// The default `[clipboard].timeout_seconds` (3.4): 45.
const fn default_clipboard_timeout_seconds() -> u64 {
    45
}

/// `[run]` (3.4).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfig {
    /// A prefix prepended to every environment variable name in `run`
    /// and `env`.
    #[serde(default)]
    pub env_prefix: String,
}

/// `[migrate]` (3.4).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrateConfig {
    /// Path or name of the `gpg` binary used to read legacy `OpenPGP`
    /// stores.
    #[serde(default = "default_gpg_binary")]
    pub gpg: String,
}

impl Default for MigrateConfig {
    fn default() -> Self {
        Self {
            gpg: default_gpg_binary(),
        }
    }
}

/// The default `[migrate].gpg` (3.4): `"gpg"`.
fn default_gpg_binary() -> String {
    "gpg".to_owned()
}

impl Config {
    /// Load the configuration file at `path`.
    ///
    /// A missing file yields [`Config::default`]. Any other read failure,
    /// or a file that does not parse as TOML matching this shape
    /// (unknown keys included), is an error whose message names `path`
    /// and, for a parse failure, the offending key.
    ///
    /// # Errors
    ///
    /// Returns an error if `path` exists but cannot be read, or if its
    /// contents are not a valid configuration document.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(err) => {
                return Err(err).with_context(|| format!("reading config file {}", path.display()));
            }
        };
        toml::from_str::<Self>(&contents)
            .map_err(|err| anyhow::anyhow!("invalid config file {}: {err}", path.display()))
    }
}

/// Expand a leading `~` in `raw` to `home`. Nothing else expands (3.4).
#[must_use]
pub fn expand_tilde(raw: &str, home: &Path) -> PathBuf {
    raw.strip_prefix("~/").map_or_else(
        || {
            if raw == "~" {
                home.to_path_buf()
            } else {
                PathBuf::from(raw)
            }
        },
        |rest| home.join(rest),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{Config, expand_tilde};
    use std::path::Path;

    #[test]
    fn missing_file_yields_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let config = Config::load(&path).expect("load");
        assert_eq!(config.clipboard.timeout_seconds, 45);
        assert_eq!(config.migrate.gpg, "gpg");
        assert!(config.identity.files.is_empty());
    }

    #[test]
    fn unknown_key_is_an_error_naming_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[identity]\nnope = true\n").expect("write");
        let err = Config::load(&path).expect_err("should fail");
        assert!(err.to_string().contains("nope"));
    }

    #[test]
    fn expand_tilde_prefix_only() {
        let home = Path::new("/home/t");
        assert_eq!(
            expand_tilde("~/.ssh/id_ed25519", home),
            home.join(".ssh/id_ed25519")
        );
        assert_eq!(expand_tilde("~", home), home);
        assert_eq!(expand_tilde("/abs/path", home), Path::new("/abs/path"));
        assert_eq!(expand_tilde("relative", home), Path::new("relative"));
    }
}
