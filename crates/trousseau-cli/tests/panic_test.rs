//! The panic hook never leaks a secret (3.5.1): `TROUSSEAU_PASSPHRASE`
//! and a command-line argument must appear in neither stderr nor the
//! report file the hidden `__panic-test` subcommand's panic produces.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

const PASSPHRASE_SECRET: &str = "hunter2";
const ARGUMENT_SECRET: &str = "hunter3";

#[test]
fn panic_report_never_contains_a_secret() {
    let env = common::Env::new();
    let mut cmd = env.command();
    cmd.env("TROUSSEAU_PASSPHRASE", PASSPHRASE_SECRET);
    cmd.args(["__panic-test", ARGUMENT_SECRET]);
    let assert = cmd.assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();

    assert!(
        !stderr.contains(PASSPHRASE_SECRET),
        "stderr leaked the passphrase: {stderr}"
    );
    assert!(
        !stderr.contains(ARGUMENT_SECRET),
        "stderr leaked the argument: {stderr}"
    );

    let report_dir = env.cache_home().join("trousseau");
    let mut found_report = false;
    let entries = std::fs::read_dir(&report_dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", report_dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("panic-report-") {
            continue;
        }
        found_report = true;
        let contents = std::fs::read_to_string(entry.path()).expect("read report file");
        assert!(
            !contents.contains(PASSPHRASE_SECRET),
            "report file leaked the passphrase: {contents}"
        );
        assert!(
            !contents.contains(ARGUMENT_SECRET),
            "report file leaked the argument: {contents}"
        );
    }
    assert!(
        found_report,
        "expected a panic report file under {}",
        report_dir.display()
    );
}
