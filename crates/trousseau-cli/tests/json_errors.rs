//! `--json` with a failing command prints a JSON error object on
//! stderr and nothing on stdout (3.5.1).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

/// `get` on a key that does not exist, in `--json` mode: exit 5, stdout
/// stays empty, and stderr carries the JSON error object with code
/// `key_not_found` (3.5.5, 3.5.1). Every subcommand is implemented as
/// of step 3.9, so this uses an ordinary failure instead of the
/// "not implemented yet" stub earlier steps relied on here.
#[test]
fn json_mode_error_prints_json_error_only() {
    let env = common::Env::new();
    env.init_store();

    let assert = env
        .command_with_identity()
        .args(["--json", "get", "missing-key"])
        .assert()
        .failure()
        .code(5);
    let output = assert.get_output();
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty, got: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr: serde_json::Value = serde_json::from_slice(&output.stderr).expect("stderr is JSON");
    assert_eq!(stderr["error"]["code"], "key_not_found");
    let message = stderr["error"]["message"]
        .as_str()
        .expect("error.message is a string");
    assert!(message.contains("missing-key"));
}
