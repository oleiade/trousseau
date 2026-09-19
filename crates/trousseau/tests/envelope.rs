//! Round-trip, error-mapping and tampering tests for the age envelope.
//! See `docs/IMPLEMENTATION_PLAN.md` section 3.1.1, 3.3.3 and step 2.2.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::BufReader;
use std::process::Command;
use std::str::FromStr;

use secrecy::{ExposeSecret, SecretString};
use trousseau::envelope::{
    EnvelopeKind, open_with_identities, open_with_passphrase, peek_kind, seal_to_recipients,
    seal_with_passphrase,
};
use trousseau::error::Error;

const ED25519_PUB: &str = include_str!("fixtures/ssh/id_ed25519.pub");
const ED25519_PRIV: &[u8] = include_bytes!("fixtures/ssh/id_ed25519");
const RSA_PUB: &str = include_str!("fixtures/ssh/id_rsa.pub");
const RSA_PRIV: &[u8] = include_bytes!("fixtures/ssh/id_rsa");
const ED25519_PW_PUB: &str = include_str!("fixtures/ssh/id_ed25519_pw.pub");
const ED25519_PW_PRIV: &[u8] = include_bytes!("fixtures/ssh/id_ed25519_pw");

fn x25519_identity() -> age::x25519::Identity {
    age::x25519::Identity::generate()
}

#[test]
fn round_trip_with_one_x25519_identity() {
    let identity = x25519_identity();
    let recipient: Box<dyn age::Recipient + Send> = Box::new(identity.to_public());
    let sealed = seal_to_recipients(b"hello world", &[recipient]).expect("seal succeeds");

    assert!(
        sealed.ends_with(b"\n"),
        "armored output ends with a newline"
    );
    assert!(sealed.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));

    let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity)];
    let opened = open_with_identities(&sealed, &identities).expect("open succeeds");
    assert_eq!(&opened[..], b"hello world");
}

#[test]
fn round_trip_with_two_x25519_recipients_each_identity_opens_alone() {
    let identity_a = x25519_identity();
    let identity_b = x25519_identity();
    let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![
        Box::new(identity_a.to_public()),
        Box::new(identity_b.to_public()),
    ];
    let sealed = seal_to_recipients(b"shared secret", &recipients).expect("seal succeeds");

    let identities_a: Vec<Box<dyn age::Identity>> = vec![Box::new(identity_a)];
    let opened_a = open_with_identities(&sealed, &identities_a).expect("identity a opens it");
    assert_eq!(&opened_a[..], b"shared secret");

    let identities_b: Vec<Box<dyn age::Identity>> = vec![Box::new(identity_b)];
    let opened_b = open_with_identities(&sealed, &identities_b).expect("identity b opens it");
    assert_eq!(&opened_b[..], b"shared secret");
}

#[test]
fn round_trip_with_ssh_ed25519_recipient() {
    let recipient = age::ssh::Recipient::from_str(ED25519_PUB).expect("valid ssh ed25519 pubkey");
    let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![Box::new(recipient)];
    let sealed = seal_to_recipients(b"ssh secret", &recipients).expect("seal succeeds");

    let identity = age::ssh::Identity::from_buffer(BufReader::new(ED25519_PRIV), None)
        .expect("valid ssh ed25519 private key");
    let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity)];
    let opened = open_with_identities(&sealed, &identities).expect("open succeeds");
    assert_eq!(&opened[..], b"ssh secret");
}

#[test]
fn round_trip_with_ssh_rsa_recipient() {
    let recipient = age::ssh::Recipient::from_str(RSA_PUB).expect("valid ssh rsa pubkey");
    let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![Box::new(recipient)];
    let sealed = seal_to_recipients(b"rsa secret", &recipients).expect("seal succeeds");

    let identity = age::ssh::Identity::from_buffer(BufReader::new(RSA_PRIV), None)
        .expect("valid ssh rsa private key");
    let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity)];
    let opened = open_with_identities(&sealed, &identities).expect("open succeeds");
    assert_eq!(&opened[..], b"rsa secret");
}

/// A minimal test-only `age::Callbacks` implementation that answers
/// every passphrase request with a fixed passphrase. Never used outside
/// tests: a real callback implementation belongs in `trousseau-cli`
/// (step 2.3 onward), which can prompt interactively.
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

#[test]
fn ssh_encrypted_identity_decrypts_with_a_fixed_passphrase_callback() {
    // This only exercises parsing the encrypted key fixture and
    // decrypting it through `age::Callbacks`; step 2.3 is where this
    // key is actually wired into the CLI's identity resolution, with a
    // callback implementation that prompts interactively instead of
    // returning a fixed string. It is included here because the
    // fixture is generated in this step and this is the simplest place
    // to prove it round-trips.
    let identity = age::ssh::Identity::from_buffer(BufReader::new(ED25519_PW_PRIV), None)
        .expect("valid (encrypted) ssh ed25519 private key");
    assert!(
        matches!(identity, age::ssh::Identity::Encrypted(_)),
        "id_ed25519_pw must parse as an encrypted identity"
    );
    let identity_with_callbacks = identity.with_callbacks(FixedPassphrase("test"));

    let recipient =
        age::ssh::Recipient::from_str(ED25519_PW_PUB).expect("valid ssh ed25519 pubkey");
    let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![Box::new(recipient)];
    let sealed =
        seal_to_recipients(b"encrypted ssh key secret", &recipients).expect("seal succeeds");

    let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity_with_callbacks)];
    let opened = open_with_identities(&sealed, &identities).expect("open succeeds");
    assert_eq!(&opened[..], b"encrypted ssh key secret");
}

#[test]
fn passphrase_round_trip() {
    let passphrase = SecretString::from("correct horse battery staple".to_owned());
    let sealed = seal_with_passphrase(b"passphrase secret", &passphrase).expect("seal succeeds");

    assert_eq!(
        peek_kind(&sealed).expect("valid age file"),
        EnvelopeKind::Passphrase
    );

    let opened = open_with_passphrase(&sealed, &passphrase).expect("open succeeds");
    assert_eq!(&opened[..], b"passphrase secret");
}

#[test]
fn wrong_passphrase_is_unlock_error() {
    let passphrase = SecretString::from("correct horse battery staple".to_owned());
    let wrong = SecretString::from("incorrect horse".to_owned());
    let sealed = seal_with_passphrase(b"passphrase secret", &passphrase).expect("seal succeeds");

    let err = open_with_passphrase(&sealed, &wrong).expect_err("wrong passphrase must fail");
    assert!(matches!(err, Error::Unlock { .. }), "got {err:?}");
}

#[test]
fn peek_kind_on_recipients_file_returns_recipients() {
    let identity = x25519_identity();
    let recipient: Box<dyn age::Recipient + Send> = Box::new(identity.to_public());
    let sealed = seal_to_recipients(b"data", &[recipient]).expect("seal succeeds");
    assert_eq!(
        peek_kind(&sealed).expect("valid age file"),
        EnvelopeKind::Recipients
    );
}

#[test]
fn peek_kind_on_garbage_is_invalid_store() {
    let err = peek_kind(b"hello").expect_err("garbage is not an age file");
    assert!(matches!(err, Error::InvalidStore { .. }), "got {err:?}");
}

#[test]
fn peek_kind_on_unarmored_binary_age_file_is_invalid_store() {
    let identity = x25519_identity();
    let recipient_ref: &dyn age::Recipient = &identity.to_public();
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(recipient_ref)).expect("recipient set");
    let mut binary = Vec::new();
    let mut writer = encryptor.wrap_output(&mut binary).expect("wrap output");
    std::io::Write::write_all(&mut writer, b"data").expect("write plaintext");
    writer.finish().expect("finish stream");

    assert!(!binary.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));
    let err = peek_kind(&binary).expect_err("binary age files are not accepted stores");
    assert!(matches!(err, Error::InvalidStore { .. }), "got {err:?}");
}

#[test]
fn tampering_with_the_armored_body_fails_to_open() {
    let identity = x25519_identity();
    let recipient: Box<dyn age::Recipient + Send> = Box::new(identity.to_public());
    let sealed = seal_to_recipients(b"tamper me", &[recipient]).expect("seal succeeds");

    let text = String::from_utf8(sealed).expect("armor is ascii");
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    assert!(
        lines.len() >= 3,
        "armored output must have header, body and footer lines"
    );

    // Flip one byte in a body line: not the first (header) line, and
    // not the last non-empty line (the `-----END...-----` footer).
    let last_body_index = lines.len() - 2;
    let target = lines
        .iter()
        .enumerate()
        .skip(1)
        .take(last_body_index)
        .find(|(_, line)| !line.is_empty())
        .map(|(index, _)| index)
        .expect("at least one non-empty body line between header and footer");

    let mut bytes = lines[target].clone().into_bytes();
    let flip_at = bytes.len() / 2;
    bytes[flip_at] ^= 0x01;
    lines[target] = String::from_utf8(bytes).expect("flipping one bit stays ascii");

    let mut tampered = lines.join("\n").into_bytes();
    tampered.push(b'\n');

    let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity)];
    let err = open_with_identities(&tampered, &identities)
        .expect_err("tampering must not decrypt successfully");
    // Flipping a base64 character can corrupt either the armor encoding
    // itself (surfacing while the header/MAC is parsed, an
    // `InvalidStore`) or just the ciphertext (surfacing during
    // decryption, an `Unlock`). Observed in this suite: the flipped
    // line falls inside the header/MAC portion of the armor body, so
    // `Decryptor::new_buffered` itself fails and this is consistently
    // `Error::InvalidStore`.
    assert!(
        matches!(err, Error::Unlock { .. } | Error::InvalidStore { .. }),
        "got {err:?}"
    );
}

/// True if `age` or `rage` is on `PATH`.
fn age_binary_on_path() -> Option<&'static str> {
    ["age", "rage"]
        .into_iter()
        .find(|candidate| Command::new(candidate).arg("--version").output().is_ok())
}

#[test]
#[allow(clippy::print_stdout)]
fn interop_with_system_age_binary() {
    let Some(binary) = age_binary_on_path() else {
        println!("skipped: age binary not found");
        return;
    };

    let identity = x25519_identity();
    let recipient: Box<dyn age::Recipient + Send> = Box::new(identity.to_public());
    let sealed = seal_to_recipients(b"interop payload", &[recipient]).expect("seal succeeds");

    let identity_dir = tempfile::tempdir().expect("temp dir");
    let identity_path = identity_dir.path().join("identity.txt");
    std::fs::write(
        &identity_path,
        format!("{}\n", identity.to_string().expose_secret()),
    )
    .expect("write identity file");

    let output = Command::new(binary)
        .args(["-d", "-i"])
        .arg(&identity_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write as _;
            child
                .stdin
                .take()
                .expect("piped stdin")
                .write_all(&sealed)?;
            child.wait_with_output()
        })
        .expect("run system age binary");

    assert!(output.status.success(), "age -d failed: {output:?}");
    assert_eq!(output.stdout, b"interop payload");
}
