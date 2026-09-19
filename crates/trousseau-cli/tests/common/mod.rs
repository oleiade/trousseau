//! Shared helpers for `trousseau-cli` integration tests.
//!
//! Every test builds its `trousseau` invocation through [`Env`], which
//! points `HOME` and the XDG directories (and their Windows
//! equivalents) at a fresh [`tempfile::TempDir`] and clears every
//! `TROUSSEAU_*` variable, so no test ever depends on, or leaks into,
//! the developer's real environment.
//!
//! [`Env::init_store`] creates a recipients store sealed to the fixture
//! SSH identity's recipient only (`--no-self`), so a test can unlock it
//! deterministically with [`ssh_identity_path`] instead of depending on
//! a freshly generated identity, per the phase 3 preamble in
//! `docs/IMPLEMENTATION_PLAN.md`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// The fixture SSH `ed25519` public key used across `trousseau-cli`'s
/// integration tests, shared with `crates/trousseau`'s own tests.
pub const SSH_PUB: &str = include_str!("../../../trousseau/tests/fixtures/ssh/id_ed25519.pub");

/// The matching private key file's path, passed directly to
/// `--identity`.
#[must_use]
pub fn ssh_identity_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../trousseau/tests/fixtures/ssh/id_ed25519")
}

/// Parse a `trousseau ... --json` command's stdout as JSON.
#[allow(clippy::expect_used)]
#[must_use]
pub fn json_stdout(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

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

    /// Create a project store (3.2) at [`Env::path`]'s `.trousseau`,
    /// sealed to the fixture SSH identity's recipient only (`--no-self`).
    ///
    /// Every command run against a store created this way must pass
    /// `--identity` pointed at [`ssh_identity_path`] to unlock it; see
    /// [`Env::command_with_identity`].
    pub fn init_store(&self) {
        self.command()
            .args(["init", "--recipient", SSH_PUB.trim(), "--no-self"])
            .assert()
            .success();
    }

    /// [`Env::command`], with `--identity` already pointed at the fixture
    /// SSH identity [`Env::init_store`] sealed the store to.
    #[must_use]
    pub fn command_with_identity(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("--identity").arg(ssh_identity_path());
        cmd
    }

    /// The project store's path: `.trousseau` under [`Env::path`] (3.2).
    #[must_use]
    pub fn store_path(&self) -> PathBuf {
        self.path().join(".trousseau")
    }

    /// The directory advisory lock files live in (3.2):
    /// `<cache>/trousseau/locks`.
    #[must_use]
    pub fn lock_dir(&self) -> PathBuf {
        self.cache.join("trousseau").join("locks")
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
