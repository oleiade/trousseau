//! Tests for recipient parsing/normalization and identity loading. See
//! `docs/IMPLEMENTATION_PLAN.md` section 3.3 and step 2.3.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::str::FromStr;

use secrecy::{ExposeSecret, SecretString};
use time::macros::datetime;
use trousseau::envelope::{open_with_identities, seal_to_recipients, seal_with_passphrase};
use trousseau::error::Error;
use trousseau::identity::{
    IdentityFileKind, RecipientKind, detect_identity_file, generate_identity, load_identities,
    normalize_recipients, own_recipients, parse_recipient, same_recipient, to_age_recipients,
};

const X25519_RECIPIENT: &str = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";
const ED25519_PUB: &str = include_str!("fixtures/ssh/id_ed25519.pub");
const ED25519_PRIV: &[u8] = include_bytes!("fixtures/ssh/id_ed25519");
const RSA_PUB: &str = include_str!("fixtures/ssh/id_rsa.pub");
const ED25519_PW_PUB: &str = include_str!("fixtures/ssh/id_ed25519_pw.pub");
const ED25519_PW_PRIV: &[u8] = include_bytes!("fixtures/ssh/id_ed25519_pw");

/// A real ECDSA SSH public key (from `age`'s own test suite), used only
/// to prove `parse_recipient` rejects it by key type.
const ECDSA_PUB: &str = "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBHFliOyIZs1gxGF3fmDxFykQhE88wy6AKDGFBfn0R6ZuvRmENABZQa9+pj9hMki+LX0qDJbmHTiWDbYv/cmFt/Q=";

/// A synthetic (but structurally valid) `sk-ssh-ed25519@openssh.com`
/// public key, built by hand: the SSH wire encoding of just the key
/// type string, base64-encoded. `age` only needs the type tag to reject
/// it, so the rest of the key body is irrelevant.
const SK_ED25519_PUB: &str = "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29t";

/// A structurally valid (correct Bech32 checksum) plugin recipient for
/// a plugin named `yubikey`, with arbitrary payload bytes. Used only to
/// prove `to_age_recipients` looks for `age-plugin-yubikey` on `PATH`
/// and fails cleanly when it is not there; no plugin ever runs.
const VALID_YUBIKEY_PLUGIN_RECIPIENT: &str =
    "age1yubikey1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0s9hkmc0";

/// A minimal test-only `age::Callbacks` implementation that answers
/// every passphrase request with a fixed passphrase. Mirrors the
/// `FixedPassphrase` helper in `tests/envelope.rs`: a real
/// implementation that prompts interactively belongs in
/// `trousseau-cli`.
#[derive(Clone)]
struct FixedPassphrase(&'static str);

impl age::Callbacks for FixedPassphrase {
    fn display_message(&self, _message: &str) {}

    fn confirm(&self, _message: &str, _yes_string: &str, _no_string: Option<&str>) -> Option<bool> {
        Some(true)
    }

    fn request_public_string(&self, _description: &str) -> Option<String> {
        None
    }

    fn request_passphrase(&self, _description: &str) -> Option<SecretString> {
        Some(SecretString::from(self.0.to_owned()))
    }
}

/// Split a fixture's `"<type> <base64> <comment>"` public key line into
/// `(type, base64)`, dropping the comment.
fn ssh_key_and_material(pubkey: &str) -> (&str, &str) {
    let mut parts = pubkey.trim_end().splitn(3, ' ');
    let key_type = parts.next().expect("key type");
    let material = parts.next().expect("key material");
    (key_type, material)
}

// ---- parse_recipient ----

#[test]
fn parse_x25519_recipient() {
    let parsed = parse_recipient(X25519_RECIPIENT).expect("valid x25519 recipient");
    assert_eq!(parsed.kind, RecipientKind::X25519);
    assert_eq!(parsed.canonical, X25519_RECIPIENT);
    assert_eq!(parsed.original, X25519_RECIPIENT);
}

#[test]
fn parse_ssh_ed25519_recipient() {
    let input = ED25519_PUB.trim_end();
    let parsed = parse_recipient(input).expect("valid ssh ed25519 recipient");
    assert_eq!(parsed.kind, RecipientKind::SshEd25519);
    assert!(parsed.canonical.starts_with("ssh-ed25519 "));
    assert!(
        !parsed.canonical.contains("trousseau-test"),
        "canonical form must drop the comment: {}",
        parsed.canonical
    );
    assert_eq!(parsed.original, input);
}

#[test]
fn parse_ssh_rsa_recipient() {
    let input = RSA_PUB.trim_end();
    let parsed = parse_recipient(input).expect("valid ssh rsa recipient");
    assert_eq!(parsed.kind, RecipientKind::SshRsa);
    assert!(parsed.canonical.starts_with("ssh-rsa "));
    assert!(!parsed.canonical.contains("trousseau-test"));
}

#[test]
fn parse_plugin_recipient() {
    let parsed = parse_recipient("age1yubikey1abc").expect("looks like a plugin recipient");
    assert_eq!(parsed.kind, RecipientKind::Plugin("yubikey".to_owned()));
    assert_eq!(parsed.canonical, "age1yubikey1abc");
    assert_eq!(parsed.original, "age1yubikey1abc");
}

#[test]
fn parse_rejects_ecdsa_ssh_key() {
    let err = parse_recipient(ECDSA_PUB).expect_err("ecdsa keys are rejected");
    match err {
        Error::InvalidRecipient { reason, .. } => {
            assert!(reason.contains("ecdsa-sha2-nistp256"), "got {reason:?}");
        }
        other => panic!("expected InvalidRecipient, got {other:?}"),
    }
}

#[test]
fn parse_rejects_sk_ssh_key() {
    let err = parse_recipient(SK_ED25519_PUB).expect_err("sk- (FIDO/U2F) keys are rejected");
    match err {
        Error::InvalidRecipient { reason, .. } => {
            assert!(
                reason.contains("sk-ssh-ed25519@openssh.com"),
                "got {reason:?}"
            );
        }
        other => panic!("expected InvalidRecipient, got {other:?}"),
    }
}

#[test]
fn parse_rejects_an_identity_string() {
    let identity_like = "AGE-SECRET-KEY-1QQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQ";
    let err = parse_recipient(identity_like).expect_err("an identity string is not a recipient");
    assert!(matches!(err, Error::InvalidRecipient { .. }));
}

#[test]
fn parse_rejects_garbage() {
    let err = parse_recipient("hello").expect_err("garbage is not a recipient");
    assert!(matches!(err, Error::InvalidRecipient { .. }));
}

// ---- normalize_recipients / same_recipient ----

#[test]
fn normalize_recipients_dedupes_by_key_material_keeps_first_and_sorts() {
    let (key_type, material) = ssh_key_and_material(ED25519_PUB);
    let with_comment_a = format!("{key_type} {material} alice@laptop");
    let with_comment_b = format!("{key_type} {material} bob@desktop");

    let normalized = normalize_recipients(vec![
        X25519_RECIPIENT.to_owned(),
        with_comment_a.clone(),
        with_comment_b,
    ])
    .expect("all entries are valid recipients");

    assert_eq!(normalized.len(), 2, "the ssh key collapses to one entry");
    assert!(
        normalized.contains(&with_comment_a),
        "the first occurrence's original string (with its comment) is kept"
    );
    assert!(!normalized.iter().any(|r| r.contains("bob@desktop")));

    let mut sorted_by_canonical = normalized.clone();
    sorted_by_canonical.sort_by_key(|s| parse_recipient(s).expect("valid").canonical);
    assert_eq!(
        normalized, sorted_by_canonical,
        "result must already be sorted by canonical form"
    );
}

#[test]
fn normalize_recipients_rejects_invalid_entry() {
    let err = normalize_recipients(vec!["hello".to_owned()]).expect_err("invalid entry errors");
    assert!(matches!(err, Error::InvalidRecipient { .. }));
}

#[test]
fn same_recipient_ignores_ssh_comment_but_not_key_material() {
    let (key_type, material) = ssh_key_and_material(ED25519_PUB);
    let a = format!("{key_type} {material} alice@laptop");
    let b = format!("{key_type} {material} bob@desktop");
    assert!(same_recipient(&a, &b));
    assert!(!same_recipient(X25519_RECIPIENT, &a));
    assert!(!same_recipient("hello", &a));
}

// ---- detect_identity_file ----

#[test]
fn detect_identity_file_kinds() {
    assert_eq!(
        detect_identity_file(b"# a comment\nAGE-SECRET-KEY-1QQQQQQQQQQQQQQQQQ\n"),
        Some(IdentityFileKind::AgePlain)
    );
    assert_eq!(
        detect_identity_file(
            b"-----BEGIN AGE ENCRYPTED FILE-----\nYWdl\n-----END AGE ENCRYPTED FILE-----\n"
        ),
        Some(IdentityFileKind::AgeEncrypted)
    );
    assert_eq!(
        detect_identity_file(ED25519_PRIV),
        Some(IdentityFileKind::Ssh)
    );
    assert_eq!(
        detect_identity_file(b"AGE-PLUGIN-YUBIKEY-1QQQQQQQQQQQQQQQQQ\n"),
        Some(IdentityFileKind::Plugin)
    );
    assert_eq!(detect_identity_file(b"nothing recognizable here\n"), None);
}

// ---- to_age_recipients ----

#[test]
fn to_age_recipients_builds_x25519_and_ssh_recipients() {
    let list = vec![
        X25519_RECIPIENT.to_owned(),
        ED25519_PUB.trim_end().to_owned(),
    ];
    let recipients =
        to_age_recipients(&list, FixedPassphrase("unused")).expect("builds both recipients");
    assert_eq!(recipients.len(), 2);
}

#[test]
fn to_age_recipients_missing_plugin_binary_is_invalid_recipient() {
    // No `age-plugin-yubikey` binary exists in this test environment;
    // this must surface as an error naming the binary, and must never
    // attempt to run one.
    let Err(err) = to_age_recipients(
        &[VALID_YUBIKEY_PLUGIN_RECIPIENT.to_owned()],
        FixedPassphrase("unused"),
    ) else {
        panic!("no such plugin binary");
    };
    match err {
        Error::InvalidRecipient { reason, .. } => {
            assert!(reason.contains("age-plugin-yubikey"), "got {reason:?}");
        }
        other => panic!("expected InvalidRecipient, got {other:?}"),
    }
}

// ---- load_identities ----

#[test]
fn load_identities_plain_age_file_with_two_identities_and_a_comment() {
    let identity_a = age::x25519::Identity::generate();
    let identity_b = age::x25519::Identity::generate();
    let contents = format!(
        "# a comment\n{}\n{}\n",
        identity_a.to_string().expose_secret(),
        identity_b.to_string().expose_secret()
    );
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("identity.txt");
    std::fs::write(&path, contents).expect("write identity file");

    let identities =
        load_identities(&[path], FixedPassphrase("unused")).expect("loads both identities");
    assert_eq!(identities.len(), 2);
}

#[test]
fn load_identities_ssh_ed25519_unencrypted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("id_ed25519");
    std::fs::write(&path, ED25519_PRIV).expect("write key");

    let identities =
        load_identities(&[path], FixedPassphrase("unused")).expect("loads the identity");
    assert_eq!(identities.len(), 1);

    let recipient = age::ssh::Recipient::from_str(ED25519_PUB.trim_end()).expect("valid recipient");
    let sealed = seal_to_recipients(b"ssh secret", &[Box::new(recipient)]).expect("seal succeeds");
    let opened = open_with_identities(&sealed, &identities).expect("decrypts");
    assert_eq!(&opened[..], b"ssh secret");
}

#[test]
fn load_identities_ssh_ed25519_encrypted_with_callbacks() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("id_ed25519_pw");
    std::fs::write(&path, ED25519_PW_PRIV).expect("write key");

    let identities = load_identities(&[path], FixedPassphrase("test")).expect("loads the identity");
    assert_eq!(identities.len(), 1);

    let recipient =
        age::ssh::Recipient::from_str(ED25519_PW_PUB.trim_end()).expect("valid recipient");
    let sealed =
        seal_to_recipients(b"encrypted ssh secret", &[Box::new(recipient)]).expect("seal succeeds");
    let opened = open_with_identities(&sealed, &identities)
        .expect("decrypts using the passphrase from callbacks");
    assert_eq!(&opened[..], b"encrypted ssh secret");
}

#[test]
fn load_identities_encrypted_age_identity_file() {
    let generated = generate_identity(datetime!(2026-09-19 00:00:00 UTC));
    let passphrase = SecretString::from("correct horse battery staple".to_owned());
    let sealed = seal_with_passphrase(
        generated.identity_file_contents.expose_secret().as_bytes(),
        &passphrase,
    )
    .expect("seal succeeds");

    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("identity.txt.age");
    std::fs::write(&path, &sealed).expect("write encrypted identity file");

    let identities = load_identities(&[path], FixedPassphrase("correct horse battery staple"))
        .expect("loads the encrypted identity file");
    assert_eq!(identities.len(), 1);

    let recipient =
        age::x25519::Recipient::from_str(&generated.recipient).expect("valid recipient");
    let store_sealed =
        seal_to_recipients(b"payload", &[Box::new(recipient)]).expect("seal succeeds");
    let opened = open_with_identities(&store_sealed, &identities)
        .expect("decrypts after prompting for the identity file's own passphrase");
    assert_eq!(&opened[..], b"payload");
}

#[test]
fn load_identities_garbage_file_is_invalid_identity() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("garbage.txt");
    std::fs::write(&path, b"this is not an identity file\n").expect("write garbage");

    let Err(err) = load_identities(&[path], FixedPassphrase("unused")) else {
        panic!("garbage is rejected");
    };
    assert!(matches!(err, Error::InvalidIdentity { .. }));
}

#[test]
fn load_identities_missing_file_is_io_error() {
    let missing = PathBuf::from("/nonexistent/trousseau-test-path/identity.txt");
    let Err(err) = load_identities(&[missing], FixedPassphrase("unused")) else {
        panic!("missing file errors");
    };
    assert!(matches!(err, Error::Io(_)));
}

// ---- generate_identity ----

#[test]
fn generate_identity_output_parses_back_and_its_recipient_opens_what_it_sealed() {
    let generated = generate_identity(datetime!(2026-09-19 12:00:00 UTC));
    let contents = generated.identity_file_contents.expose_secret().to_owned();
    assert!(contents.starts_with("# created: "));
    assert!(contents.contains("# public key: "));
    assert!(contents.contains(&generated.recipient));
    assert!(contents.contains("AGE-SECRET-KEY-1"));
    assert!(contents.ends_with('\n'));

    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("identity.txt");
    std::fs::write(&path, &contents).expect("write generated identity file");

    let identities = load_identities(&[path], FixedPassphrase("unused"))
        .expect("the generated identity file parses back through load_identities");
    assert_eq!(identities.len(), 1);

    let recipient =
        age::x25519::Recipient::from_str(&generated.recipient).expect("valid recipient");
    let sealed = seal_to_recipients(b"round trip", &[Box::new(recipient)]).expect("seal succeeds");
    let opened =
        open_with_identities(&sealed, &identities).expect("its recipient opens what it sealed");
    assert_eq!(&opened[..], b"round trip");
}

#[test]
fn generated_identity_debug_redacts_the_secret_key() {
    let generated = generate_identity(datetime!(2026-09-19 12:00:00 UTC));
    let rendered = format!("{generated:?}");
    assert!(!rendered.contains("AGE-SECRET-KEY-1"));
    assert!(rendered.contains(&generated.recipient));
}

// ---- own_recipients ----

#[test]
fn own_recipients_covers_plain_age_and_unencrypted_ssh_only() {
    let dir = tempfile::tempdir().expect("temp dir");

    let generated = generate_identity(datetime!(2026-09-19 12:00:00 UTC));
    let age_path = dir.path().join("identity.txt");
    std::fs::write(&age_path, generated.identity_file_contents.expose_secret())
        .expect("write age identity");

    let ssh_path = dir.path().join("id_ed25519");
    std::fs::write(&ssh_path, ED25519_PRIV).expect("write ssh key");

    let ssh_encrypted_path = dir.path().join("id_ed25519_pw");
    std::fs::write(&ssh_encrypted_path, ED25519_PW_PRIV).expect("write encrypted ssh key");

    let recipients = own_recipients(&[age_path, ssh_path, ssh_encrypted_path]);

    assert!(recipients.contains(&generated.recipient));
    assert!(
        recipients
            .iter()
            .any(|r| same_recipient(r, ED25519_PUB.trim_end())),
        "the unencrypted ssh key's recipient must be present"
    );
    assert_eq!(
        recipients.len(),
        2,
        "the encrypted ssh key contributes nothing (best effort, unencrypted only)"
    );
}

#[test]
fn own_recipients_never_errors_on_unreadable_paths() {
    let missing = PathBuf::from("/nonexistent/trousseau-test-path/identity.txt");
    assert_eq!(own_recipients(&[missing]), Vec::<String>::new());
}
