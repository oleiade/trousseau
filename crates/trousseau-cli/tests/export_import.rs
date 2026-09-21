//! `trousseau export` and `import` (3.5.11, step 3.5).
//!
//! Every test creates a recipients store with [`common::Env::init_store`],
//! sealed to the fixture SSH identity's recipient only, and unlocks it
//! through [`common::Env::command_with_identity`].
//!
//! The `export --format toml` / `document::from_toml` cross-test (the
//! toml half of Step 3.5's test list) reuses `src/document.rs` directly
//! via `#[path]`, since `trousseau-cli` is a binary-only crate with no
//! library target for an integration test to depend on; `edit` (which
//! also uses this format) is step 3.7's own cross-test.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[path = "../src/document.rs"]
mod document;

use common::json_stdout;

/// A `TROUSSEAU_TEST_NOW` value earlier than [`LATER`] (debug-only, 5.2).
const EARLIER: &str = "2026-01-01T00:00:00Z";
/// A `TROUSSEAU_TEST_NOW` value later than [`EARLIER`].
const LATER: &str = "2026-01-02T00:00:00Z";

/// `export` (json) of a store round-trips through
/// `import --strategy overwrite` into a fresh store: values, encodings,
/// env, and description are equal; `created_at` is preserved from the
/// document; `updated_at` is set to now.
#[test]
fn export_json_round_trips_through_import_overwrite() {
    let src = common::Env::new();
    src.init_store();
    src.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args([
            "set",
            "database/password",
            "--env",
            "DATABASE_PASSWORD",
            "--description",
            "Postgres app role",
        ])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let bin_path = src.path().join("server.key");
    std::fs::write(&bin_path, [0x00, 0x01, 0xff]).expect("write binary fixture");
    src.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", EARLIER)
        .args(["set", "tls/server.key", "--from-file"])
        .arg(&bin_path)
        .assert()
        .success();

    let export_output = src
        .command_with_identity()
        .args(["export"])
        .output()
        .expect("run export");
    assert!(export_output.status.success());

    let src_password = json_stdout(
        &src.command_with_identity()
            .args(["get", "database/password", "--json"])
            .output()
            .expect("get"),
    );
    let src_tls = json_stdout(
        &src.command_with_identity()
            .args(["get", "tls/server.key", "--json"])
            .output()
            .expect("get"),
    );

    let dst = common::Env::new();
    dst.init_store();
    dst.command_with_identity()
        .env("TROUSSEAU_TEST_NOW", LATER)
        .args(["import", "--strategy", "overwrite"])
        .write_stdin(export_output.stdout)
        .assert()
        .success();

    let dst_password = json_stdout(
        &dst.command_with_identity()
            .args(["get", "database/password", "--json"])
            .output()
            .expect("get"),
    );
    let dst_tls = json_stdout(
        &dst.command_with_identity()
            .args(["get", "tls/server.key", "--json"])
            .output()
            .expect("get"),
    );

    assert_eq!(dst_password["value"], src_password["value"]);
    assert_eq!(dst_password["encoding"], src_password["encoding"]);
    assert_eq!(dst_password["env"], src_password["env"]);
    assert_eq!(dst_password["description"], src_password["description"]);
    assert_eq!(dst_password["created_at"], src_password["created_at"]);
    assert_eq!(dst_password["created_at"], "2026-01-01T00:00:00Z");
    assert_eq!(dst_password["updated_at"], "2026-01-02T00:00:00Z");

    assert_eq!(dst_tls["value"], src_tls["value"]);
    assert_eq!(dst_tls["encoding"], "base64");
    assert_eq!(dst_tls["created_at"], "2026-01-01T00:00:00Z");
    assert_eq!(dst_tls["updated_at"], "2026-01-02T00:00:00Z");
}

/// `export --format dotenv` escapes `"`, `\`, and a newline; a
/// `base64` entry is skipped with a stderr warning.
#[test]
fn export_dotenv_escapes_and_skips_binary_with_warning() {
    let env = common::Env::new();
    env.init_store();

    let value_path = env.path().join("value.txt");
    // Bytes verbatim, via `--from-file`: a literal `"`, `\`, and a
    // newline in the middle of the value.
    std::fs::write(&value_path, b"a\"b\\c\nd").expect("write value fixture");
    env.command_with_identity()
        .args(["set", "greeting", "--env", "GREETING", "--from-file"])
        .arg(&value_path)
        .assert()
        .success();

    let bin_path = env.path().join("secret.bin");
    std::fs::write(&bin_path, [0x00, 0x01, 0xff]).expect("write binary fixture");
    env.command_with_identity()
        .args(["set", "secretbin", "--from-file"])
        .arg(&bin_path)
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .args(["export", "--format", "dotenv"])
        .output()
        .expect("run export");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    let escaped = "a\\\"b\\\\c\\nd";
    let expected_line = format!("GREETING=\"{escaped}\"\n");
    assert!(
        stdout.contains(&expected_line),
        "stdout did not contain the expected escaped line: {stdout:?}"
    );
    assert!(!stdout.contains("secretbin"));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("skipping binary entry secretbin"));
}

/// `import --format dotenv` from a file with comments, blank lines,
/// single and double quotes, unquoted values, and `export NAME=value`
/// lines (the `export ` prefix is accepted).
#[test]
fn import_dotenv_parses_comments_quotes_and_export_prefix() {
    let env = common::Env::new();
    env.init_store();

    let dotenv_path = env.path().join("import.env");
    std::fs::write(
        &dotenv_path,
        concat!(
            "# a comment\n",
            "\n",
            "FOO=bar\n",
            "export BAZ=qux\n",
            "QUUX=\"hello \\\"world\\\"\"\n",
            "SINGLE='raw $value'\n",
        ),
    )
    .expect("write dotenv fixture");

    env.command_with_identity()
        .args(["import", "--format", "dotenv"])
        .arg(&dotenv_path)
        .assert()
        .success();

    let foo = json_stdout(
        &env.command_with_identity()
            .args(["get", "foo", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(foo["value"], "bar");
    assert_eq!(foo["env"], "FOO");

    let baz = json_stdout(
        &env.command_with_identity()
            .args(["get", "baz", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(baz["value"], "qux");
    assert_eq!(baz["env"], "BAZ");

    let quux = json_stdout(
        &env.command_with_identity()
            .args(["get", "quux", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(quux["value"], "hello \"world\"");
    assert_eq!(quux["env"], "QUUX");

    let single = json_stdout(
        &env.command_with_identity()
            .args(["get", "single", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(single["value"], "raw $value");
    assert_eq!(single["env"], "SINGLE");
}

/// A minimal, hand-built `--format json` import document with two
/// entries: `shared` (collides with a pre-existing key in every test
/// below) and `new` (does not).
fn two_entry_json_document() -> String {
    concat!(
        "{",
        r#""schema":1,"kind":"passphrase","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z","recipients":[],"#,
        r#""entries":{"#,
        r#""shared":{"value":"imported","encoding":"utf8","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},"#,
        r#""new":{"value":"newval","encoding":"utf8","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#,
        "}}"
    )
    .to_owned()
}

/// `import`'s default strategy, `fail`, exits 8 on collision and writes
/// nothing: neither the colliding key nor any other key in the same
/// import is applied.
#[test]
fn import_default_strategy_fail_exits_8_and_writes_nothing() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "shared"])
        .write_stdin("orig\n")
        .assert()
        .success();

    env.command_with_identity()
        .args(["import"])
        .write_stdin(two_entry_json_document())
        .assert()
        .failure()
        .code(8);

    let shared = json_stdout(
        &env.command_with_identity()
            .args(["get", "shared", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(shared["value"], "orig");

    env.command_with_identity()
        .args(["get", "new"])
        .assert()
        .failure()
        .code(5);
}

/// `import --strategy keep` leaves the existing entry alone and applies
/// only the non-colliding one.
#[test]
fn import_strategy_keep_leaves_existing_entry() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "shared"])
        .write_stdin("orig\n")
        .assert()
        .success();

    let assert = env
        .command_with_identity()
        .args(["import", "--strategy", "keep", "--json"])
        .write_stdin(two_entry_json_document())
        .assert()
        .success();
    let json = json_stdout(assert.get_output());
    assert_eq!(json["ok"], true);
    assert_eq!(json["added"], 1);
    assert_eq!(json["updated"], 0);
    assert_eq!(json["skipped"], 1);

    let shared = json_stdout(
        &env.command_with_identity()
            .args(["get", "shared", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(shared["value"], "orig");

    let new = json_stdout(
        &env.command_with_identity()
            .args(["get", "new", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(new["value"], "newval");
}

/// `import --strategy overwrite` replaces the existing entry and
/// applies the non-colliding one.
#[test]
fn import_strategy_overwrite_replaces_existing_entry() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "shared"])
        .write_stdin("orig\n")
        .assert()
        .success();

    let assert = env
        .command_with_identity()
        .args(["import", "--strategy", "overwrite", "--json"])
        .write_stdin(two_entry_json_document())
        .assert()
        .success();
    let json = json_stdout(assert.get_output());
    assert_eq!(json["ok"], true);
    assert_eq!(json["added"], 1);
    assert_eq!(json["updated"], 1);
    assert_eq!(json["skipped"], 0);

    let shared = json_stdout(
        &env.command_with_identity()
            .args(["get", "shared", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(shared["value"], "imported");

    let new = json_stdout(
        &env.command_with_identity()
            .args(["get", "new", "--json"])
            .output()
            .expect("get"),
    );
    assert_eq!(new["value"], "newval");
}

/// `export --out` refuses to overwrite an existing file without
/// `--force` (exit 8), and warns on stderr that the file holds
/// plaintext secrets.
#[test]
fn export_out_refuses_overwrite_without_force_and_warns() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args(["set", "k"])
        .write_stdin("v\n")
        .assert()
        .success();

    let out_path = env.path().join("out.json");
    let first = env
        .command_with_identity()
        .args(["export", "--out"])
        .arg(&out_path)
        .output()
        .expect("run export --out");
    assert!(first.status.success());
    let stderr = String::from_utf8(first.stderr).expect("stderr is utf8");
    assert!(stderr.contains(&format!(
        "{} contains plaintext secrets",
        out_path.display()
    )));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&out_path)
            .expect("stat out file")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    env.command_with_identity()
        .args(["export", "--out"])
        .arg(&out_path)
        .assert()
        .failure()
        .code(8);

    env.command_with_identity()
        .args(["export", "--out"])
        .arg(&out_path)
        .arg("--force")
        .assert()
        .success();
}

/// `export --format toml` output parses with `document::from_toml`
/// (the `edit` document format, 3.5.12): the cross-test with `edit`
/// itself lands in step 3.7.
#[test]
fn export_toml_output_parses_with_document_from_toml() {
    let env = common::Env::new();
    env.init_store();
    env.command_with_identity()
        .args([
            "set",
            "database/password",
            "--env",
            "DATABASE_PASSWORD",
            "--description",
            "Postgres app role",
        ])
        .write_stdin("s3cr3t\n")
        .assert()
        .success();

    let bin_path = env.path().join("server.key");
    std::fs::write(&bin_path, [0x00, 0x01, 0xff]).expect("write binary fixture");
    env.command_with_identity()
        .args(["set", "tls/server.key", "--from-file"])
        .arg(&bin_path)
        .assert()
        .success();

    let output = env
        .command_with_identity()
        .args(["export", "--format", "toml"])
        .output()
        .expect("run export");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("stdout is utf8");

    let parsed = document::from_toml(&text).expect("parse toml export");
    assert_eq!(parsed.len(), 2);

    let password_key: trousseau::schema::Key = "database/password".parse().expect("parse key");
    let password = &parsed[&password_key];
    assert_eq!(password.value.expose(), b"s3cr3t");
    assert_eq!(password.encoding, trousseau::schema::Encoding::Utf8);
    assert_eq!(password.env.as_deref(), Some("DATABASE_PASSWORD"));
    assert_eq!(password.description.as_deref(), Some("Postgres app role"));

    let tls_key: trousseau::schema::Key = "tls/server.key".parse().expect("parse key");
    let tls = &parsed[&tls_key];
    assert_eq!(tls.value.expose(), &[0x00, 0x01, 0xff]);
    assert_eq!(tls.encoding, trousseau::schema::Encoding::Base64);
}
