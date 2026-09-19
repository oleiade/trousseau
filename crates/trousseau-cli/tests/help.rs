//! `--help` lists every subcommand (3.5), snapshotted with `insta`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

#[test]
fn help_lists_every_subcommand() {
    let env = common::Env::new();
    let output = env
        .command()
        .arg("--help")
        .output()
        .expect("run trousseau --help");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    insta::assert_snapshot!(stdout);
}
