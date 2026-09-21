//! `trousseau __clip-clear` (3.5.16, hidden), and the clipboard-copy
//! helper behind `get --clip` (3.5.5).
//!
//! Copying a value spawns a detached copy of this binary running
//! `__clip-clear <sha256-hex-of-value> <seconds>`: it sleeps, reads the
//! clipboard back, and clears it only if the clipboard's own SHA-256
//! still matches what was copied. The value itself never appears on
//! that child's command line, only its hash (3.5.16).

use trousseau::schema::Encoding;

use crate::cli::ClipClearArgs;
use crate::context::Context;

/// Run `__clip-clear`: sleep for `args.seconds`, then clear the
/// clipboard only if its current text still hashes to `args.hash`.
///
/// Nobody observes this detached process's stdout, stderr, or exit
/// code (its stdio is redirected to null and nothing waits on it), so
/// there is nothing gained by swallowing a clipboard failure here
/// instead of just letting `?` report it; either way, a failure simply
/// means the clipboard is left as it was.
///
/// # Errors
///
/// Returns an error if the clipboard cannot be opened, its current
/// content cannot be read as text, or (on a hash match) it cannot be
/// cleared.
#[cfg(feature = "clipboard")]
pub fn run(_ctx: &Context, args: &ClipClearArgs) -> anyhow::Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(args.seconds));
    let mut clipboard = arboard::Clipboard::new()?;
    let current = clipboard.get_text()?;
    if hash_matches(&current, &args.hash) {
        clipboard.clear()?;
    }
    Ok(())
}

/// Run `__clip-clear` (built without the `clipboard` feature): a no-op
/// that always succeeds (step 3.9 acceptance).
///
/// The `Result` return type is dictated by [`super::dispatch`], which
/// every subcommand handler, this one included, must match.
///
/// # Errors
///
/// Never returns an error.
#[cfg(not(feature = "clipboard"))]
#[allow(clippy::unnecessary_wraps)]
pub const fn run(_ctx: &Context, _args: &ClipClearArgs) -> anyhow::Result<()> {
    Ok(())
}

/// Copy `bytes` (an entry's raw value, stored as `encoding`) to the
/// clipboard for `get --clip` (3.5.5), print the `copied KEY to
/// clipboard, clearing in <N>s` notice on stderr, and, unless the
/// configured timeout is `0`, spawn the detached `__clip-clear` that
/// clears it later (3.5.16).
///
/// The clipboard text is the value's UTF-8 text for a `utf8` entry, or
/// its standard base64 encoding for a `base64` entry: the same "stored
/// representation" `get --json` reports, always valid text a clipboard
/// can hold.
///
/// # Errors
///
/// Returns an error if the system clipboard cannot be opened or written
/// to, or if the detached clearing process cannot be spawned.
#[cfg(feature = "clipboard")]
pub fn copy_to_clipboard(
    ctx: &Context,
    key: &str,
    encoding: Encoding,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let text = clipboard_text(encoding, bytes);
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text.clone())?;

    let seconds = ctx.config.clipboard.timeout_seconds;
    crate::output::info(
        ctx.quiet,
        &format!("copied {key} to clipboard, clearing in {seconds}s"),
    );
    if seconds > 0 {
        spawn_clear(&sha256_hex(text.as_bytes()), seconds)?;
    }
    Ok(())
}

/// Copy to the clipboard (built without the `clipboard` feature): always
/// fails with "built without clipboard support" (3.5.5).
///
/// # Errors
///
/// Always returns an error.
#[cfg(not(feature = "clipboard"))]
pub fn copy_to_clipboard(
    _ctx: &Context,
    _key: &str,
    _encoding: Encoding,
    _bytes: &[u8],
) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("built without clipboard support"))
}

/// The clipboard text for an entry's `encoding` and raw `bytes`: the
/// same "stored representation" `get --json` reports (3.1.2, 3.5.5).
#[cfg(feature = "clipboard")]
fn clipboard_text(encoding: Encoding, bytes: &[u8]) -> String {
    match encoding {
        Encoding::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Encoding::Base64 => {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(bytes)
        }
    }
}

/// Spawn a detached `trousseau __clip-clear <hash> <seconds>` (3.5.16):
/// stdio redirected to null, and, on Unix, in its own process group
/// (`setsid`-like) so it outlives this process; on Windows, in its own
/// process group and detached from this process's console.
///
/// # Errors
///
/// Returns an error if the current executable's path cannot be
/// determined, or if the child process cannot be spawned.
#[cfg(feature = "clipboard")]
fn spawn_clear(hash: &str, seconds: u64) -> anyhow::Result<()> {
    use anyhow::Context as _;

    let exe = std::env::current_exe().context("locating the current executable")?;
    let mut command = std::process::Command::new(exe);
    command
        .arg("__clip-clear")
        .arg(hash)
        .arg(seconds.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        /// `CREATE_NEW_PROCESS_GROUP` (`winbase.h`): detach the child
        /// from this process's console control group.
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        /// `DETACHED_PROCESS` (`winbase.h`): the child gets no console
        /// at all.
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }

    command
        .spawn()
        .context("spawning the clipboard-clearing process")?;
    Ok(())
}

/// Whether `current_text`'s SHA-256 equals `expected_hash` (3.5.16),
/// comparing case-insensitively since a hash is opaque hex, not a value
/// whose case carries meaning.
#[cfg(feature = "clipboard")]
fn hash_matches(current_text: &str, expected_hash: &str) -> bool {
    sha256_hex(current_text.as_bytes()).eq_ignore_ascii_case(expected_hash)
}

/// The lowercase hex SHA-256 digest of `bytes` (3.5.16): the same
/// digest `get --clip` passes to `__clip-clear` on its command line.
#[cfg(feature = "clipboard")]
fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        // Writing to a `String` through `fmt::Write` never fails.
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use clap::{CommandFactory as _, Parser as _};

    use crate::cli::{Cli, Command};

    /// `__clip-clear <hash> <seconds>` parses its two positional
    /// arguments in order, and never as flags (a hex hash and a bare
    /// number both happen to look nothing like a flag, but this pins
    /// the shape down).
    #[test]
    fn clip_clear_parses_hash_and_seconds() {
        let cli =
            Cli::try_parse_from(["trousseau", "__clip-clear", "deadbeef", "5"]).expect("parses");
        let Command::ClipClear(args) = cli.command else {
            unreachable!("__clip-clear should parse to Command::ClipClear")
        };
        assert_eq!(args.hash, "deadbeef");
        assert_eq!(args.seconds, 5);
    }

    /// `__clip-clear` is hidden from `--help` (`cli.rs` sets `hide =
    /// true` on it), not absent from the grammar: it is a real
    /// subcommand `main.rs` can dispatch to.
    #[test]
    fn clip_clear_is_hidden_but_parseable() {
        let subcommand = Cli::command()
            .find_subcommand("__clip-clear")
            .expect("declared as a subcommand")
            .clone();
        assert!(subcommand.is_hide_set());
        assert!(Cli::try_parse_from(["trousseau", "__clip-clear", "abc", "0"]).is_ok());
    }
}

#[cfg(all(test, feature = "clipboard"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod clipboard_tests {
    use super::{hash_matches, sha256_hex};

    // Reference digests from `printf '<input>' | shasum -a 256`.
    const SHA256_EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn sha256_hex_matches_known_vectors() {
        assert_eq!(sha256_hex(b""), SHA256_EMPTY);
        assert_eq!(sha256_hex(b"abc"), SHA256_ABC);
    }

    #[test]
    fn hash_matches_is_case_insensitive_and_content_sensitive() {
        assert!(hash_matches("abc", SHA256_ABC));
        assert!(hash_matches("abc", &SHA256_ABC.to_uppercase()));
        assert!(!hash_matches("abcd", SHA256_ABC));
        assert!(!hash_matches("abc", SHA256_EMPTY));
    }
}
