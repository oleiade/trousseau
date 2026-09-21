//! `--help` lists every subcommand (3.5), snapshotted with `insta`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

/// Every command path whose `--help` (and, for a few, `-h`) output this
/// crate snapshot-tests, so a change to help text shows up as a
/// reviewable diff instead of silently rotting.
const PATHS: &[&[&str]] = &[
    &["init"],
    &["info"],
    &["set"],
    &["get"],
    &["ls"],
    &["rm"],
    &["mv"],
    &["recipients"],
    &["recipients", "ls"],
    &["recipients", "add"],
    &["recipients", "rm"],
    &["rekey"],
    &["export"],
    &["import"],
    &["run"],
    &["env"],
    &["edit"],
    &["migrate"],
    &["completions"],
    &["man"],
];

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

/// Every subcommand's `--help`, snapshotted so help text changes show up
/// in review.
#[test]
fn every_subcommand_help_is_snapshotted() {
    let env = common::Env::new();
    for path in PATHS {
        let output = env
            .command()
            .args(*path)
            .arg("--help")
            .output()
            .expect("run trousseau <path> --help");
        assert!(output.status.success(), "{path:?}");
        assert!(output.stderr.is_empty(), "{path:?}");
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        insta::assert_snapshot!(format!("help_{}", path.join("_")), stdout);
    }
}

/// `-h` (the short summary) for the three commands most likely to be
/// typed without `--help`: it must still say where a value comes from
/// and how selection works, not just list flag names.
#[test]
fn short_help_for_common_commands_is_snapshotted() {
    let env = common::Env::new();
    for path in [["set"].as_slice(), ["run"].as_slice(), ["env"].as_slice()] {
        let output = env
            .command()
            .args(path)
            .arg("-h")
            .output()
            .expect("run trousseau <path> -h");
        assert!(output.status.success(), "{path:?}");
        assert!(output.stderr.is_empty(), "{path:?}");
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        insta::assert_snapshot!(format!("short_help_{}", path.join("_")), stdout);
    }
}
