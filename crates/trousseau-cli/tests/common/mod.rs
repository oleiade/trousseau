//! Shared helpers for `trousseau-cli` integration tests.
//!
//! Every test builds its `trousseau` invocation through [`Env`], which
//! points `HOME` and the XDG directories (and their Windows
//! equivalents) at a fresh [`tempfile::TempDir`] and clears every
//! `TROUSSEAU_*` variable, so no test ever depends on, or leaks into,
//! the developer's real environment.
//!
//! Step 3.1 has no `init` yet, so this helper only sets up the
//! environment; a later step (3.2) is expected to grow a
//! `Env::init_store` (or similar) that also creates a store with a
//! known identity, per the phase 3 preamble in
//! `docs/IMPLEMENTATION_PLAN.md`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// An isolated environment for one or more `trousseau` invocations.
pub struct Env {
    dir: TempDir,
    home: PathBuf,
    config: PathBuf,
    data: PathBuf,
    cache: PathBuf,
}

impl Env {
    /// Create a fresh temporary directory and the `home`, `config`,
    /// `data`, and `cache` subdirectories every test environment
    /// variable points at.
    #[allow(clippy::expect_used)]
    #[must_use]
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("create temp dir");
        let home = dir.path().join("home");
        let config = dir.path().join("config");
        let data = dir.path().join("data");
        let cache = dir.path().join("cache");
        for path in [&home, &config, &data, &cache] {
            std::fs::create_dir_all(path).expect("create env subdirectory");
        }
        Self {
            dir,
            home,
            config,
            data,
            cache,
        }
    }

    /// The temporary directory's root. Also used as the current
    /// directory for every command this `Env` builds, so project-store
    /// discovery (3.2) starts from a known, empty location.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// The fake `HOME`.
    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The fake `XDG_CONFIG_HOME`.
    #[must_use]
    pub fn config_home(&self) -> &Path {
        &self.config
    }

    /// The directory the CLI resolves as its data root: the fake
    /// `XDG_DATA_HOME` on Unix, and the fake `APPDATA` (which etcetera
    /// uses for both config and data) on Windows.
    #[must_use]
    pub fn data_home(&self) -> &Path {
        #[cfg(windows)]
        {
            &self.config
        }
        #[cfg(not(windows))]
        {
            &self.data
        }
    }

    /// The fake `XDG_CACHE_HOME`.
    #[must_use]
    pub fn cache_home(&self) -> &Path {
        &self.cache
    }

    /// Build a `trousseau` [`Command`] with this environment applied.
    ///
    /// The command's current directory is [`Env::path`]; `HOME`,
    /// `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and `XDG_CACHE_HOME` (and, on
    /// Windows, `APPDATA`, `LOCALAPPDATA`, and `USERPROFILE`) point at
    /// this environment's subdirectories; every `TROUSSEAU_*` variable
    /// inherited from the test process is removed.
    #[allow(clippy::expect_used)]
    #[must_use]
    pub fn command(&self) -> Command {
        let mut cmd = Command::cargo_bin("trousseau").expect("binary should build");
        cmd.current_dir(self.path());
        for (key, _) in std::env::vars() {
            if key.starts_with("TROUSSEAU_") {
                cmd.env_remove(key);
            }
        }
        cmd.env("HOME", &self.home);
        cmd.env("XDG_CONFIG_HOME", &self.config);
        cmd.env("XDG_DATA_HOME", &self.data);
        cmd.env("XDG_CACHE_HOME", &self.cache);
        #[cfg(windows)]
        {
            // etcetera's Windows strategy reads config AND data from
            // `APPDATA` and the cache from `LOCALAPPDATA`.
            cmd.env("APPDATA", &self.config);
            cmd.env("LOCALAPPDATA", &self.cache);
            cmd.env("USERPROFILE", &self.home);
        }
        cmd
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
