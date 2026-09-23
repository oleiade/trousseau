//! Recipients and identities.
//!
//! This module implements `docs/IMPLEMENTATION_PLAN.md` section 3.3:
//! parsing recipient strings, normalizing recipient lists, and loading
//! identity sources. It is the only place in the crate (besides
//! [`crate::envelope`]) that talks to the `age` crate directly. It
//! defines no prompting: the [`age::Callbacks`] implementor that
//! actually asks a human for a passphrase lives in `trousseau-cli`;
//! this module only ever receives one and hands it to `age`.

use std::collections::BTreeMap;
use std::fmt;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use secrecy::{ExposeSecret, SecretString};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::envelope::ARMOR_HEADER;
use crate::error::Error;

/// The maximum scrypt work factor accepted when opening an
/// age-encrypted identity file (3.3.3): caps a hostile header from
/// pinning the CPU indefinitely.
const MAX_IDENTITY_WORK_FACTOR: u8 = 22;

/// Which of the three accepted recipient forms a [`ParsedRecipient`] is
/// (3.3.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecipientKind {
    /// A native age X25519 recipient (`age1...`).
    X25519,
    /// An `ssh-ed25519` public key reused as a recipient.
    SshEd25519,
    /// An `ssh-rsa` public key reused as a recipient.
    SshRsa,
    /// A plugin recipient (`age1<plugin>1...`), naming the plugin.
    Plugin(String),
}

/// The result of successfully parsing a recipient string (3.3.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedRecipient {
    /// Which kind of recipient this is.
    pub kind: RecipientKind,
    /// The canonical form used for deduplication and sorting: an X25519
    /// recipient's Bech32 string lowercased, an SSH recipient's
    /// `"<type> <base64>"` without its comment, or a plugin recipient
    /// unchanged.
    pub canonical: String,
    /// The original string, exactly as given (including an SSH
    /// comment, if any).
    pub original: String,
}

/// Parse and validate a recipient string (3.3.1).
///
/// Accepts an age X25519 recipient, an `ssh-ed25519` or `ssh-rsa`
/// public key (with an optional trailing comment), or a plugin
/// recipient (`age1<plugin>1...`). ECDSA and FIDO/U2F (`sk-`) SSH keys
/// are recognized but rejected, with a message naming the key type.
///
/// # Errors
///
/// Returns [`Error::InvalidRecipient`] if `input` is not one of the
/// accepted forms.
pub fn parse_recipient(input: &str) -> Result<ParsedRecipient, Error> {
    if input.is_empty() {
        return Err(Error::InvalidRecipient {
            input: input.to_owned(),
            reason: "recipient must not be empty".to_owned(),
        });
    }

    if let Ok(recipient) = age::x25519::Recipient::from_str(input) {
        return Ok(ParsedRecipient {
            kind: RecipientKind::X25519,
            canonical: recipient.to_string().to_lowercase(),
            original: input.to_owned(),
        });
    }

    match age::ssh::Recipient::from_str(input) {
        Ok(recipient) => {
            let kind = match &recipient {
                age::ssh::Recipient::SshEd25519(..) => RecipientKind::SshEd25519,
                age::ssh::Recipient::SshRsa(..) => RecipientKind::SshRsa,
            };
            return Ok(ParsedRecipient {
                kind,
                canonical: recipient.to_string(),
                original: input.to_owned(),
            });
        }
        Err(err) => {
            if let Some(reason) = ssh_rejection_reason(&err) {
                return Err(Error::InvalidRecipient {
                    input: input.to_owned(),
                    reason,
                });
            }
            // Not an SSH key at all: fall through and try the other forms.
        }
    }

    if let Some(name) = plugin_name(input) {
        return Ok(ParsedRecipient {
            kind: RecipientKind::Plugin(name),
            canonical: input.to_owned(),
            original: input.to_owned(),
        });
    }

    if input.starts_with("AGE-SECRET-KEY-1") {
        return Err(Error::InvalidRecipient {
            input: input.to_owned(),
            reason: "this is an identity, not a recipient".to_owned(),
        });
    }

    Err(Error::InvalidRecipient {
        input: input.to_owned(),
        reason: "not a recognized age, SSH, or plugin recipient".to_owned(),
    })
}

/// Turn an SSH recipient parse failure into a rejection reason, or
/// `None` if the input simply was not an SSH key at all (so the caller
/// should keep trying other recipient forms).
fn ssh_rejection_reason(err: &age::ssh::ParseRecipientKeyError) -> Option<String> {
    match err {
        age::ssh::ParseRecipientKeyError::Unsupported(key_type) => {
            Some(format!("unsupported SSH key type {key_type}"))
        }
        age::ssh::ParseRecipientKeyError::RsaModulusTooLarge => {
            Some("ssh-rsa modulus too large".to_owned())
        }
        age::ssh::ParseRecipientKeyError::RsaModulusTooSmall => {
            Some("ssh-rsa modulus too small (minimum 2048 bits)".to_owned())
        }
        _ => None,
    }
}

/// If `input` looks like a plugin recipient (`age1<name>1...`), return
/// the plugin name. This is a lightweight structural check (splitting
/// on the last `1`, per the Bech32 HRP/data separator), not a full
/// Bech32 checksum validation: [`to_age_recipients`] does that when it
/// actually builds the plugin recipient.
fn plugin_name(input: &str) -> Option<String> {
    let separator = input.rfind('1')?;
    let hrp = &input[..separator];
    let name = hrp.strip_prefix("age1")?;
    (!name.is_empty() && name.bytes().all(is_plugin_name_byte)).then(|| name.to_owned())
}

/// `true` for the characters `age`'s plugin names are allowed to
/// contain: ASCII alphanumerics, and `+ - . _`.
const fn is_plugin_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.' | b'_')
}

/// Build `age` recipients ready for [`crate::envelope::seal_to_recipients`].
///
/// Plugin recipients are resolved lazily here: building a
/// [`age::plugin::RecipientPluginV1`] requires the plugin binary
/// (`age-plugin-<name>`) to be on `PATH`.
///
/// # Errors
///
/// Returns [`Error::InvalidRecipient`] if any entry of `list` is not a
/// valid recipient string, or if a plugin recipient's binary cannot be
/// found on `PATH`.
// `callbacks` is a fixed part of this step's public API (an owned,
// `Clone` value: `age::plugin::RecipientPluginV1::new` needs its own
// owned instance per plugin), even though a list with no plugin
// recipients never consumes it.
#[allow(clippy::needless_pass_by_value)]
pub fn to_age_recipients(
    list: &[String],
    callbacks: impl age::Callbacks,
) -> Result<Vec<Box<dyn age::Recipient + Send>>, Error> {
    let mut recipients: Vec<Box<dyn age::Recipient + Send>> = Vec::with_capacity(list.len());
    for raw in list {
        let parsed = parse_recipient(raw)?;
        match parsed.kind {
            RecipientKind::X25519 => {
                let recipient = age::x25519::Recipient::from_str(raw).map_err(|reason| {
                    Error::InvalidRecipient {
                        input: raw.clone(),
                        reason: reason.to_owned(),
                    }
                })?;
                recipients.push(Box::new(recipient));
            }
            RecipientKind::SshEd25519 | RecipientKind::SshRsa => {
                let recipient =
                    age::ssh::Recipient::from_str(raw).map_err(|err| Error::InvalidRecipient {
                        input: raw.clone(),
                        reason: ssh_rejection_reason(&err)
                            .unwrap_or_else(|| "invalid SSH recipient".to_owned()),
                    })?;
                recipients.push(Box::new(recipient));
            }
            RecipientKind::Plugin(name) => {
                let plugin_recipient = age::plugin::Recipient::from_str(raw).map_err(|reason| {
                    Error::InvalidRecipient {
                        input: raw.clone(),
                        reason: reason.to_owned(),
                    }
                })?;
                let plugin = age::plugin::RecipientPluginV1::new(
                    &name,
                    std::slice::from_ref(&plugin_recipient),
                    &[],
                    callbacks.clone(),
                )
                .map_err(|err| Error::InvalidRecipient {
                    input: raw.clone(),
                    reason: err.to_string(),
                })?;
                recipients.push(Box::new(plugin));
            }
        }
    }
    Ok(recipients)
}

/// Deduplicate `list` by key material, keep each canonical form's first
/// occurrence's original string, and return the result sorted by
/// canonical form.
///
/// # Errors
///
/// Returns [`Error::InvalidRecipient`] if any entry of `list` is not a
/// valid recipient string.
pub fn normalize_recipients(list: Vec<String>) -> Result<Vec<String>, Error> {
    let mut by_canonical: BTreeMap<String, String> = BTreeMap::new();
    for raw in list {
        let parsed = parse_recipient(&raw)?;
        by_canonical
            .entry(parsed.canonical)
            .or_insert(parsed.original);
    }
    Ok(by_canonical.into_values().collect())
}

/// `true` if `a` and `b` parse to the same recipient (same key
/// material, ignoring an SSH comment). `false` if either fails to
/// parse.
#[must_use]
pub fn same_recipient(a: &str, b: &str) -> bool {
    match (parse_recipient(a), parse_recipient(b)) {
        (Ok(left), Ok(right)) => left.canonical == right.canonical,
        _ => false,
    }
}

/// Which kind of identity source a file's bytes are (3.3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityFileKind {
    /// An age identity file: lines starting with `AGE-SECRET-KEY-1`
    /// and `#` comments.
    AgePlain,
    /// An armored age file whose decrypted payload is an age identity
    /// file.
    AgeEncrypted,
    /// An OpenSSH private key.
    Ssh,
    /// An age plugin identity file.
    Plugin,
}

/// Detect which kind of identity source `bytes` are, by prefix (3.3.2).
///
/// Returns `None` if `bytes` matches none of the four recognized forms.
#[must_use]
pub fn detect_identity_file(bytes: &[u8]) -> Option<IdentityFileKind> {
    if bytes.starts_with(ARMOR_HEADER) {
        return Some(IdentityFileKind::AgeEncrypted);
    }
    if bytes.starts_with(b"-----BEGIN OPENSSH PRIVATE KEY-----") {
        return Some(IdentityFileKind::Ssh);
    }
    for line in bytes.split(|&b| b == b'\n') {
        if line.starts_with(b"AGE-SECRET-KEY-1") {
            return Some(IdentityFileKind::AgePlain);
        }
        if line.starts_with(b"AGE-PLUGIN-") {
            return Some(IdentityFileKind::Plugin);
        }
    }
    None
}

/// Read and parse every identity file in `paths` (3.3.2).
///
/// `callbacks` answers any passphrase or plugin prompt. Identities from
/// every path are returned together, in order; `age` tries each in
/// turn when opening a store.
///
/// # Errors
///
/// Returns [`Error::Io`] if a path cannot be read, and
/// [`Error::InvalidIdentity`] if a path's contents do not match one of
/// the four recognized identity file kinds, or match a kind but fail to
/// parse as one.
// `callbacks` is a fixed part of this step's public API (an owned,
// `Clone` value: nested `age` APIs such as `IdentityFile::with_callbacks`
// and `encrypted::Identity::from_buffer` each need their own owned
// instance), even though not every match arm consumes it.
#[allow(clippy::needless_pass_by_value)]
pub fn load_identities(
    paths: &[PathBuf],
    callbacks: impl age::Callbacks,
) -> Result<Vec<Box<dyn age::Identity>>, Error> {
    let mut identities: Vec<Box<dyn age::Identity>> = Vec::new();
    for path in paths {
        let bytes = std::fs::read(path).map_err(Error::Io)?;
        match detect_identity_file(&bytes) {
            Some(IdentityFileKind::AgePlain | IdentityFileKind::Plugin) => {
                let file = age::IdentityFile::from_buffer(bytes.as_slice())
                    .map_err(|err| invalid_identity(path, err))?
                    .with_callbacks(callbacks.clone());
                let loaded = file
                    .into_identities()
                    .map_err(|err| invalid_identity(path, err))?;
                identities.extend(
                    loaded
                        .into_iter()
                        .map(|identity| -> Box<dyn age::Identity> { identity }),
                );
            }
            Some(IdentityFileKind::AgeEncrypted) => {
                let filename = path.to_string_lossy().into_owned();
                let armored = age::armor::ArmoredReader::new(Cursor::new(bytes));
                let identity = age::encrypted::Identity::from_buffer(
                    armored,
                    Some(filename),
                    callbacks.clone(),
                    Some(MAX_IDENTITY_WORK_FACTOR),
                )
                .map_err(|err| invalid_identity(path, err))?
                .ok_or_else(|| Error::InvalidIdentity {
                    path: path.clone(),
                    reason: "not encrypted to a passphrase".to_owned(),
                })?;
                identities.push(Box::new(identity));
            }
            Some(IdentityFileKind::Ssh) => {
                let filename = path.to_string_lossy().into_owned();
                let identity = age::ssh::Identity::from_buffer(bytes.as_slice(), Some(filename))
                    .map_err(|err| invalid_identity(path, err))?;
                identities.push(Box::new(identity.with_callbacks(callbacks.clone())));
            }
            None => {
                return Err(Error::InvalidIdentity {
                    path: path.clone(),
                    reason: "unrecognized identity file format".to_owned(),
                });
            }
        }
    }
    Ok(identities)
}

/// Build an [`Error::InvalidIdentity`] from any displayable `age` parse
/// or decrypt error.
fn invalid_identity(path: &Path, reason: impl fmt::Display) -> Error {
    Error::InvalidIdentity {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
}

/// A freshly generated X25519 identity: ready-to-write identity file
/// contents, and the corresponding recipient string.
///
/// `Debug` is implemented by hand to redact
/// [`identity_file_contents`](Self::identity_file_contents), which
/// holds the secret key.
pub struct GeneratedIdentity {
    /// The identity file's contents: exactly
    /// `"# created: <rfc3339>\n# public key: <recipient>\n<AGE-SECRET-KEY-1...>\n"`.
    pub identity_file_contents: SecretString,
    /// The recipient string (`age1...`) corresponding to the generated
    /// identity.
    pub recipient: String,
}

impl fmt::Debug for GeneratedIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedIdentity")
            .field("identity_file_contents", &"<redacted>")
            .field("recipient", &self.recipient)
            .finish()
    }
}

/// Generate a fresh X25519 identity, dated `now`.
#[must_use]
pub fn generate_identity(now: OffsetDateTime) -> GeneratedIdentity {
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public().to_string();
    let created = now
        .format(&Rfc3339)
        .unwrap_or_else(|_| now.unix_timestamp().to_string());
    let secret_key_line = identity.to_string();
    let contents = format!(
        "# created: {created}\n# public key: {recipient}\n{}\n",
        secret_key_line.expose_secret()
    );
    GeneratedIdentity {
        identity_file_contents: SecretString::from(contents),
        recipient,
    }
}

/// Best-effort recipient strings for the identities in `paths`.
///
/// Only plain age identity files and unencrypted SSH private keys are
/// considered. Never errors; anything that cannot be read, is a
/// different identity file kind, or fails to parse is silently
/// skipped.
#[must_use]
pub fn own_recipients(paths: &[PathBuf]) -> Vec<String> {
    let mut recipients = Vec::new();
    for path in paths {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        match detect_identity_file(&bytes) {
            Some(IdentityFileKind::AgePlain) => {
                recipients.extend(own_recipients_from_age_plain(&bytes));
            }
            Some(IdentityFileKind::Ssh) => {
                recipients.extend(own_recipient_from_unencrypted_ssh(&bytes));
            }
            _ => {}
        }
    }
    recipients
}

/// The X25519 recipients corresponding to every `AGE-SECRET-KEY-1...`
/// line in a plain age identity file. Plugin identity lines (which have
/// no local public-key derivation) are silently skipped.
fn own_recipients_from_age_plain(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|line| line.trim().parse::<age::x25519::Identity>().ok())
        .map(|identity| identity.to_public().to_string())
        .collect()
}

/// The recipient corresponding to an unencrypted SSH private key, or
/// `None` if `bytes` is not one (encrypted and unsupported keys are
/// skipped).
fn own_recipient_from_unencrypted_ssh(bytes: &[u8]) -> Option<String> {
    let identity = age::ssh::Identity::from_buffer(bytes, None).ok()?;
    if !matches!(identity, age::ssh::Identity::Unencrypted(_)) {
        return None;
    }
    age::ssh::Recipient::try_from(identity)
        .ok()
        .map(|recipient| recipient.to_string())
}
