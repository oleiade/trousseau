//! `TROUSSEAU_CONFIG` pointing at a malformed configuration file (3.4).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

#[test]
fn unknown_config_key_exits_1_and_names_the_key() {
    let env = common::Env::new();
    let config_path = env.path().join("bad-config.toml");
    std::fs::write(&config_path, "[identity]\nnope = true\n").expect("write config");

    let mut cmd = env.command();
    cmd.env("TROUSSEAU_CONFIG", &config_path);
    cmd.arg("info");
    let assert = cmd.assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("nope"),
        "expected the offending key in stderr, got: {stderr}"
    );
}
