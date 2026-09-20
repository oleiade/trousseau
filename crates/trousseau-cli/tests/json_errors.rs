//! `--json` with an unimplemented subcommand prints a JSON error object
//! on stderr and nothing on stdout (3.5.1).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

#[test]
fn json_mode_unimplemented_subcommand_prints_json_error_only() {
    let env = common::Env::new();
    // `completions` is still unimplemented (step 3.9): `migrate` served
    // this role until step 3.8 implemented it, `edit` until step 3.7,
    // `run` until step 3.6, `export` until step 3.5, `rekey` until step
    // 3.4, and `set` before that (step 3.3).
    let assert = env
        .command()
        .args(["--json", "completions", "bash"])
        .assert()
        .failure();
    let output = assert.get_output();
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty, got: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr: serde_json::Value = serde_json::from_slice(&output.stderr).expect("stderr is JSON");
    let message = stderr["error"]["message"]
        .as_str()
        .expect("error.message is a string");
    assert!(message.contains("not implemented"));
    assert!(stderr["error"]["code"].as_str().is_some());
}
