//! `trousseau completions`, `trousseau man`, and `get --clip` (3.5.16,
//! 3.5.17, step 3.9).
//!
//! `get --clip` cannot be exercised reliably in headless CI: the
//! feature-enabled path is covered by
//! [`get_clip_copies_value_and_clears_it_locally`], `#[ignore]`d for a
//! developer to run locally with a real display/clipboard available.
//! The feature-disabled path (`--no-default-features`) is always
//! exercised by [`get_clip_without_clipboard_feature_exits_1`].

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

/// `completions <shell>` prints a non-empty script naming `trousseau`,
/// for every shell `clap_complete` supports (3.5.17).
#[test]
fn completions_print_non_empty_output_for_every_shell() {
    let env = common::Env::new();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = env
            .command()
            .args(["completions", shell])
            .output()
            .expect("run completions");
        assert!(output.status.success(), "shell: {shell}");
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        assert!(!stdout.trim().is_empty(), "shell: {shell}: empty output");
        assert!(
            stdout.contains("trousseau"),
            "shell: {shell}: output does not mention trousseau: {stdout}"
        );
    }
}

/// `man` (no subcommand) prints a roff page for the top-level command,
/// starting with a `.TH` title macro (3.5.17).
#[test]
fn man_top_level_contains_th_macro() {
    let env = common::Env::new();
    let output = env.command().arg("man").output().expect("run man");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains(".TH"), "missing .TH: {stdout}");
}

/// `man get` prints the `get` subcommand's own page, which documents
/// `--clip` (3.5.17).
#[test]
fn man_get_contains_clip_flag() {
    let env = common::Env::new();
    let output = env
        .command()
        .args(["man", "get"])
        .output()
        .expect("run man get");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains(".TH"), "missing .TH: {stdout}");
    // `clap_mangen` escapes `--clip`'s hyphens as roff's non-breaking
    // minus (`\-`), per `man`-page convention, so the literal `--clip`
    // never appears verbatim in the rendered page.
    assert!(stdout.contains("\\-\\-clip"), "missing --clip: {stdout}");
}

/// `man` on a name that is not a subcommand fails instead of silently
/// printing the top-level page.
#[test]
fn man_unknown_subcommand_fails() {
    let env = common::Env::new();
    env.command()
        .args(["man", "no-such-command"])
        .assert()
        .failure();
}

/// Without the `clipboard` feature, `get --clip` exits 1 with "built
/// without clipboard support" (3.5.5, step 3.9 acceptance), and prints
/// nothing to stdout.
#[cfg(not(feature = "clipboard"))]
#[test]
fn get_clip_without_clipboard_feature_exits_1() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let assert = env
        .command_with_identity()
        .args(["get", "k", "--clip"])
        .assert()
        .failure()
        .code(1);
    let output = assert.get_output();
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("built without clipboard support"),
        "stderr: {stderr}"
    );
}

/// End-to-end: `get --clip` copies the value to the real OS clipboard
/// and prints the "clearing in Ns" notice. Reads and writes the
/// developer's actual clipboard and needs a display/clipboard provider,
/// neither of which headless CI has, so this runs locally only.
#[cfg(feature = "clipboard")]
#[test]
#[ignore = "reads and writes the real OS clipboard; run locally, not in headless CI"]
fn get_clip_copies_value_and_clears_it_locally() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let assert = env
        .command_with_identity()
        .args(["get", "k", "--clip"])
        .assert()
        .success();
    let output = assert.get_output();
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("copied k to clipboard, clearing in"),
        "stderr: {stderr}"
    );

    let mut clipboard = arboard::Clipboard::new().expect("open clipboard");
    assert_eq!(clipboard.get_text().expect("read clipboard"), "v");
}
