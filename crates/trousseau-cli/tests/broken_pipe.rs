//! A closed stdout pipe exits quietly, like any other Unix tool, instead
//! of panicking into a `human-panic` crash report (regression test for
//! the `completions`/`man` broken-pipe panic).
//!
//! Rust ignores `SIGPIPE` at startup, so a write to a stdout whose
//! reader has already gone away normally surfaces as a recoverable
//! `io::Error` rather than killing the process. `clap_complete` and
//! `clap_mangen` both `.expect()` on that error internally, so without
//! `sigpipe::reset()` in `main`, `completions fish | head` panics.
//!
//! Dropping the child's piped stdout handle before reading anything
//! closes the pipe's read end deterministically, unlike piping through
//! `head`, which races the child's write against the buffer filling up.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::os::unix::process::ExitStatusExt as _;
use std::process::Stdio;

mod common;

/// `SIGPIPE`'s signal number on Unix (POSIX; consistent across Linux
/// and macOS, the two platforms CI runs on).
const SIGPIPE: i32 = 13;

/// `completions fish` into a reader that closes its end of the pipe
/// before reading anything is killed by `SIGPIPE`, not a panic.
#[cfg(unix)]
#[test]
fn completions_with_closed_stdout_exits_via_sigpipe_not_panic() {
    let env = common::Env::new();
    let mut cmd = env.std_command_with_identity();
    cmd.args(["completions", "fish"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("spawn completions fish");
    // Close the pipe's read end before the child writes anything: a
    // deterministic broken pipe, unlike piping through `head`, which
    // races the child's write against the pipe buffer filling up.
    drop(child.stdout.take());

    let output = child.wait_with_output().expect("wait for child");

    assert!(
        !output.status.success(),
        "expected a non-zero exit, got: {:?}",
        output.status
    );
    assert_eq!(
        output.status.signal(),
        Some(SIGPIPE),
        "expected the process to be killed by SIGPIPE, got: {:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("had a problem and crashed"),
        "human-panic crash report printed to stderr: {stderr}"
    );

    let report_dir = env.cache_home().join("trousseau");
    if let Ok(entries) = std::fs::read_dir(&report_dir) {
        for entry in entries {
            let name = entry.expect("dir entry").file_name();
            assert!(
                !name.to_string_lossy().starts_with("panic-report-"),
                "found a panic report file: {name:?}"
            );
        }
    }
}
