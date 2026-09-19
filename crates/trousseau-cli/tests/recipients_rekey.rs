//! `trousseau recipients` and `trousseau rekey` (3.5.9, 3.5.10, step 3.4).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{RSA_PUB, SSH_PUB, json_stdout, rsa_identity_path};

/// `recipients add` lets the newly added identity open the store, and
/// `recipients ls` then reports two recipients.
#[test]
fn recipients_add_lets_the_new_identity_unlock_and_ls_shows_two() {
    let env = common::Env::new();
    env.command()
        .args(["init", "--recipient", RSA_PUB.trim(), "--no-self"])
        .assert()
        .success();

    // Before the add, the SSH ed25519 identity cannot open the store.
    let locked = json_stdout(
        &env.command_with_identity()
            .args(["info", "--json"])
            .output()
            .expect("run info"),
    );
    assert_eq!(locked["locked"], true);

    env.command()
        .arg("--identity")
        .arg(rsa_identity_path())
        .args(["recipients", "add", SSH_PUB.trim()])
        .assert()
        .success();

    // After the add, the SSH identity opens the store.
    let unlocked = json_stdout(
        &env.command_with_identity()
            .args(["info", "--json"])
            .output()
            .expect("run info"),
    );
    assert_eq!(unlocked["locked"], false);

    let ls = env
        .command()
        .arg("--identity")
        .arg(rsa_identity_path())
        .args(["recipients", "ls", "--json"])
        .assert()
        .success();
    let json = json_stdout(ls.get_output());
    assert_eq!(json.as_array().expect("array").len(), 2);
}

/// Adding a recipient already present, only under a different SSH
/// comment, leaves the count unchanged and notes it on stderr.
#[test]
fn recipients_add_duplicate_with_different_comment_is_ignored_with_a_note() {
    let env = common::Env::new();
    env.init_store();

    let mut altered = SSH_PUB.trim().to_owned();
    let key_and_comment_boundary = altered.rfind(' ').expect("ssh key has a comment");
    altered.truncate(key_and_comment_boundary);
    altered.push_str(" a-different-comment");
    assert_ne!(altered, SSH_PUB.trim(), "the comment must actually change");

    let assert = env
        .command_with_identity()
        .args(["recipients", "add", &altered])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("already present"), "stderr: {stderr}");

    let ls = env
        .command_with_identity()
        .args(["recipients", "ls", "--json"])
        .assert()
        .success();
    let json = json_stdout(ls.get_output());
    assert_eq!(json.as_array().expect("array").len(), 1);
}

/// `recipients rm` refuses to remove the store's last recipient.
#[test]
fn recipients_rm_to_zero_exits_2() {
    let env = common::Env::new();
    env.init_store();

    env.command_with_identity()
        .args(["recipients", "rm", SSH_PUB.trim()])
        .assert()
        .failure()
        .code(2);
}

/// Removing the caller's own recipient is refused without `--force`
/// under `--no-input`, but `--force` allows it and the caller's own
/// identity can then no longer open the store.
#[test]
fn recipients_rm_own_requires_force_then_locks_self_out() {
    let env = common::Env::new();
    // No `--no-self`: this generates and uses the default identity, so
    // its own recipient is in the store.
    env.command().arg("init").assert().success();
    env.command()
        .args(["recipients", "add", SSH_PUB.trim()])
        .assert()
        .success();

    let own_recipient = {
        let ls = env
            .command()
            .args(["recipients", "ls", "--json"])
            .assert()
            .success();
        let json = json_stdout(ls.get_output());
        json.as_array()
            .expect("array")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .find(|r| r.starts_with("age1"))
            .expect("the default identity's own recipient is present")
    };

    env.command()
        .args(["--no-input", "recipients", "rm"])
        .arg(&own_recipient)
        .assert()
        .failure()
        .code(2);

    // Nothing was written: the store still has both recipients, and the
    // default identity can still open it.
    env.command().arg("info").assert().success();

    env.command()
        .args(["recipients", "rm"])
        .arg(&own_recipient)
        .arg("--force")
        .assert()
        .success();

    // `ls` needs a full unlock, unlike `info`: the default identity can
    // no longer decrypt the store (exit 4).
    env.command().arg("ls").assert().failure().code(4);

    // The other recipient (SSH) still opens it.
    env.command_with_identity().arg("info").assert().success();
}

/// `rekey` with no flags changes the ciphertext (a fresh file key every
/// time) while leaving the store openable and its entries intact.
#[test]
fn rekey_alone_changes_ciphertext_and_still_opens() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "a"])
        .write_stdin("v\n")
        .assert()
        .success();

    let before = std::fs::read(env.store_path()).expect("read store before rekey");

    env.command_with_identity().arg("rekey").assert().success();

    let after = std::fs::read(env.store_path()).expect("read store after rekey");
    assert_ne!(before, after, "rekey must mint a fresh file key");

    let get = env
        .command_with_identity()
        .args(["get", "a"])
        .output()
        .expect("run get");
    assert_eq!(get.stdout, b"v");
}

/// `rekey --to-passphrase` converts a recipients store to a passphrase
/// store (`recipients` then refuses with the `rekey` hint), and `rekey
/// --to-recipients` converts it back.
#[test]
fn rekey_to_passphrase_then_back_to_recipients() {
    let env = common::Env::new();
    env.init_store();

    let passphrase_file = env.path().join("new-pass.txt");
    std::fs::write(&passphrase_file, "correct horse battery staple\n")
        .expect("write passphrase file");

    env.command_with_identity()
        .args(["rekey", "--to-passphrase", "--passphrase-file"])
        .arg(&passphrase_file)
        .assert()
        .success();

    let info = env
        .command()
        .args(["info", "--json", "--passphrase-file"])
        .arg(&passphrase_file)
        .output()
        .expect("run info");
    let json = json_stdout(&info);
    assert_eq!(json["kind"], "passphrase");
    assert_eq!(json["recipients"], 0);

    let ls = env
        .command()
        .args(["recipients", "ls", "--passphrase-file"])
        .arg(&passphrase_file)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8_lossy(&ls.get_output().stderr);
    assert!(stderr.contains("passphrase store"), "stderr: {stderr}");

    // `recipients add` is refused the same way (3.5.9).
    env.command()
        .args(["recipients", "add", SSH_PUB.trim(), "--passphrase-file"])
        .arg(&passphrase_file)
        .assert()
        .failure()
        .code(1);

    env.command()
        .args([
            "rekey",
            "--to-recipients",
            SSH_PUB.trim(),
            "--passphrase-file",
        ])
        .arg(&passphrase_file)
        .assert()
        .success();

    // Converted back: the SSH identity opens it again, as a recipients
    // store, without the passphrase.
    let back = json_stdout(
        &env.command_with_identity()
            .args(["info", "--json"])
            .output()
            .expect("run info"),
    );
    assert_eq!(back["kind"], "recipients");
    assert_eq!(back["recipients"], 1);
    assert_eq!(back["locked"], false);
}
