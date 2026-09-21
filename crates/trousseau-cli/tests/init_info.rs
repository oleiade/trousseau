//! `trousseau init` and `trousseau info` (3.5.2, 3.5.3, step 3.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{SSH_PUB, json_stdout, ssh_identity_path};

#[test]
fn init_creates_store_and_default_identity() {
    let env = common::Env::new();

    let assert = env.command().arg("init").assert().success();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("created identity"), "stderr: {stderr}");
    assert!(stderr.contains("your recipient: age1"), "stderr: {stderr}");
    assert!(stderr.contains("created"), "stderr: {stderr}");

    let store_path = env.path().join(".trousseau");
    let store_bytes = std::fs::read(&store_path).expect("read store");
    assert!(store_bytes.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));

    let identity_path = env.config_home().join("trousseau").join("identity.txt");
    assert!(identity_path.is_file());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let identity_mode = std::fs::metadata(&identity_path)
            .expect("stat identity")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(identity_mode, 0o600);
        let store_mode = std::fs::metadata(&store_path)
            .expect("stat store")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(store_mode, 0o600);
    }

    let info_output = env
        .command()
        .args(["info", "--json"])
        .output()
        .expect("run info");
    assert!(info_output.status.success());
    let json = json_stdout(&info_output);
    assert_eq!(json["kind"], "recipients");
    assert_eq!(json["entries"], 0);
    assert_eq!(json["recipients"], 1);
    assert_eq!(json["locked"], false);

    // The human table's labels line up (3.5.3).
    let human_output = env.command().arg("info").output().expect("run info");
    let stdout = String::from_utf8(human_output.stdout).expect("utf8");
    assert!(stdout.contains("path:        "));
    assert!(stdout.contains("kind:        recipients"));
    assert!(stdout.contains("recipients:  1"));
    assert!(stdout.contains("entries:     0"));
}

#[test]
fn init_twice_exits_8() {
    let env = common::Env::new();
    env.command().arg("init").assert().success();
    env.command().arg("init").assert().failure().code(8);
}

#[test]
fn init_passphrase_store_reports_passphrase_kind() {
    let env = common::Env::new();
    let passphrase_file = env.path().join("passphrase.txt");
    std::fs::write(&passphrase_file, "correct horse battery staple\n").expect("write passphrase");

    env.command()
        .args(["init", "--passphrase", "--passphrase-file"])
        .arg(&passphrase_file)
        .assert()
        .success();

    let output = env
        .command()
        .args(["info", "--json", "--passphrase-file"])
        .arg(&passphrase_file)
        .output()
        .expect("run info");
    assert!(output.status.success());
    let json = json_stdout(&output);
    assert_eq!(json["kind"], "passphrase");
    assert_eq!(json["recipients"], 0);
    assert_eq!(json["locked"], false);
}

#[test]
fn init_with_ssh_recipient_reports_one_recipient_and_unlocks() {
    let env = common::Env::new();

    env.command()
        .args(["init", "--recipient", SSH_PUB.trim(), "--no-self"])
        .assert()
        .success();

    let output = env
        .command()
        .args(["info", "--json", "--identity"])
        .arg(ssh_identity_path())
        .output()
        .expect("run info");
    assert!(output.status.success());
    let json = json_stdout(&output);
    assert_eq!(json["locked"], false);
    assert_eq!(json["recipients"], 1);
    assert_eq!(json["kind"], "recipients");
}

#[test]
fn init_no_self_without_recipients_exits_2() {
    let env = common::Env::new();
    env.command()
        .args(["init", "--no-self"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn init_global_writes_to_data_dir_not_cwd() {
    let env = common::Env::new();
    env.command().args(["init", "--global"]).assert().success();

    let personal_store = env.data_home().join("trousseau").join("default.trousseau");
    assert!(personal_store.is_file());
    assert!(!env.path().join(".trousseau").exists());
}

#[test]
fn init_no_input_does_not_offer_existing_ssh_key() {
    let env = common::Env::new();
    let ssh_dir = env.home().join(".ssh");
    std::fs::create_dir_all(&ssh_dir).expect("create .ssh");
    std::fs::write(ssh_dir.join("id_ed25519.pub"), SSH_PUB).expect("write ssh pub");

    env.command()
        .args(["init", "--no-input"])
        .assert()
        .success();

    let output = env
        .command()
        .args(["info", "--json"])
        .output()
        .expect("run info");
    let json = json_stdout(&output);
    // Only the freshly generated default identity's own recipient: the
    // fake home's SSH key is never offered outside an interactive
    // terminal (3.5.2).
    assert_eq!(json["recipients"], 1);
}

#[test]
fn info_on_locked_store_exits_0_with_locked_true() {
    let env = common::Env::new();
    env.command()
        .args(["init", "--recipient", SSH_PUB.trim(), "--no-self"])
        .assert()
        .success();

    // No `--identity` given, and the fake HOME has no SSH key either:
    // nothing can unlock this store.
    let assert = env.command().args(["info", "--json"]).assert().success();
    let json = json_stdout(assert.get_output());
    assert_eq!(json["locked"], true);
    assert_eq!(json["kind"], "recipients");
    assert!(json["schema"].is_null());
    assert!(json["recipients"].is_null());
    assert!(json["entries"].is_null());
    assert!(json["updated_at"].is_null());

    // Same in human mode: exit 0, `(locked)` fields.
    let human = env.command().arg("info").assert().success();
    let stdout = String::from_utf8(human.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("schema:      (locked)"));
}
