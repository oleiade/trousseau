//! Integration test for `trousseau --version`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;

#[test]
fn version_flag_prints_expected_string() {
    let mut cmd = Command::cargo_bin("trousseau").expect("binary should build");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout("trousseau 1.0.0-alpha.1\n");
}
