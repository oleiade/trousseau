//! The age envelope: sealing and opening store bytes.
//!
//! A store's bytes are an ASCII-armored age file (section 3.1.1 of
//! `docs/IMPLEMENTATION_PLAN.md`). This module is the only place in the
//! crate that talks to the `age` crate directly. It has two sealing
//! entry points (to a set of recipients, or to a passphrase) and two
//! opening entry points (with a set of identities, or with a
//! passphrase), plus [`peek_kind`](crate::envelope::peek_kind) to
//! distinguish the two without decrypting.

use std::io::{Read, Write};

use age::armor::{ArmoredReader, ArmoredWriter, Format};
use age::{DecryptError, Decryptor, Encryptor};
use secrecy::SecretString;
use zeroize::Zeroizing;

use crate::error::Error;
use crate::schema::MAX_PAYLOAD_BYTES;

/// The age armor header that every trousseau store MUST start with.
///
/// Binary (unarmored) age files are not accepted as stores (section
/// 3.1.1): this crate checks for the marker before attempting to parse
/// anything, so an unarmored age file is rejected the same way as
/// arbitrary garbage. `pub(crate)` so the other byte-sniffing checks in
/// this crate (store classification in `store.rs`, identity file
/// detection in `identity.rs`) share it instead of keeping copies.
pub(crate) const ARMOR_HEADER: &[u8] = b"-----BEGIN AGE ENCRYPTED FILE-----";

/// Which kind of age envelope a store's bytes are, distinguished by the
/// age header alone (no decryption required).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeKind {
    /// One or more recipient stanzas (X25519, SSH, or plugin). The
    /// scrypt stanza is never present alongside these.
    Recipients,
    /// Exactly one scrypt (passphrase) stanza.
    Passphrase,
}

impl EnvelopeKind {
    /// This kind's lowercase name, matching [`crate::schema::StoreKind`]'s
    /// own (the two enums agree on every variant, since an envelope's
    /// kind and its decrypted store's kind describe the same store).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recipients => "recipients",
            Self::Passphrase => "passphrase",
        }
    }
}

/// Inspect the age header of `armored` without decrypting the payload.
///
/// `armored` MUST be an ASCII-armored age file (section 3.1.1); a
/// binary age file or anything else is rejected with
/// [`Error::InvalidStore`].
///
/// # Errors
///
/// Returns [`Error::InvalidStore`] if `armored` is not a valid armored
/// age file.
pub fn peek_kind(armored: &[u8]) -> Result<EnvelopeKind, Error> {
    let decryptor = new_decryptor(armored)?;
    Ok(if decryptor.is_scrypt() {
        EnvelopeKind::Passphrase
    } else {
        EnvelopeKind::Recipients
    })
}

/// Seal `plaintext` to `recipients`, producing an ASCII-armored age
/// file ending with a newline.
///
/// # Errors
///
/// Returns [`Error::Io`] if the `age` crate refuses the recipient set
/// (for example, an empty slice) or if writing the in-memory buffer
/// fails.
pub fn seal_to_recipients(
    plaintext: &[u8],
    recipients: &[Box<dyn age::Recipient + Send>],
) -> Result<Vec<u8>, Error> {
    let refs = recipients
        .iter()
        .map(|recipient| &**recipient as &dyn age::Recipient);
    let encryptor = Encryptor::with_recipients(refs).map_err(|err| io_error(&err))?;
    seal(plaintext, encryptor)
}

/// Seal `plaintext` to a passphrase, producing an ASCII-armored age
/// file ending with a newline.
///
/// Uses [`age::scrypt::Recipient`] with the crate's default work
/// factor (targeting about one second of work on the machine that
/// seals it).
///
/// # Errors
///
/// Returns [`Error::Io`] if the `age` crate refuses the recipient or if
/// writing the in-memory buffer fails.
pub fn seal_with_passphrase(plaintext: &[u8], passphrase: &SecretString) -> Result<Vec<u8>, Error> {
    let recipient = age::scrypt::Recipient::new(passphrase.clone());
    let recipient_ref: &dyn age::Recipient = &recipient;
    let encryptor =
        Encryptor::with_recipients(std::iter::once(recipient_ref)).map_err(|err| io_error(&err))?;
    seal(plaintext, encryptor)
}

/// Open `armored` with the first identity in `identities` that matches
/// one of its recipient stanzas.
///
/// The decrypted payload is capped at
/// [`MAX_PAYLOAD_BYTES`] bytes; exceeding
/// it is [`Error::TooLarge`]. The returned buffer is zeroized on drop.
///
/// # Errors
///
/// Returns [`Error::InvalidStore`] if `armored` is not a valid armored
/// age file, [`Error::Unlock`] if no identity matches or the store is
/// otherwise undecryptable, and [`Error::TooLarge`] if the decrypted
/// payload exceeds the size cap.
pub fn open_with_identities(
    armored: &[u8],
    identities: &[Box<dyn age::Identity>],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    let decryptor = new_decryptor(armored)?;
    let refs = identities
        .iter()
        .map(|identity| &**identity as &dyn age::Identity);
    let mut reader = decryptor
        .decrypt(refs)
        .map_err(|err| map_decrypt_error(&err))?;
    read_capped(&mut reader)
}

/// Open `armored` with a passphrase.
///
/// Uses [`age::scrypt::Identity`] with the maximum accepted work
/// factor capped at `2^22`, so a hostile header cannot pin the CPU
/// indefinitely.
///
/// # Errors
///
/// Returns [`Error::InvalidStore`] if `armored` is not a valid armored
/// age file, [`Error::Unlock`] if the passphrase is wrong or the
/// declared work factor exceeds the cap, and [`Error::TooLarge`] if
/// the decrypted payload exceeds the size cap.
pub fn open_with_passphrase(
    armored: &[u8],
    passphrase: &SecretString,
) -> Result<Zeroizing<Vec<u8>>, Error> {
    let decryptor = new_decryptor(armored)?;
    let mut identity = age::scrypt::Identity::new(passphrase.clone());
    identity.set_max_work_factor(22);
    let identity_ref: &dyn age::Identity = &identity;
    let mut reader = decryptor
        .decrypt(std::iter::once(identity_ref))
        .map_err(|err| map_decrypt_error(&err))?;
    read_capped(&mut reader)
}

/// Construct a `Decryptor` from armored bytes, requiring the armor
/// header to be present. Any parse failure is [`Error::InvalidStore`].
fn new_decryptor(armored: &[u8]) -> Result<Decryptor<impl std::io::BufRead + '_>, Error> {
    if !armored.starts_with(ARMOR_HEADER) {
        return Err(Error::InvalidStore {
            reason: "not an age file".to_owned(),
        });
    }
    let reader = ArmoredReader::new(armored);
    Decryptor::new_buffered(reader).map_err(|_err| Error::InvalidStore {
        reason: "not an age file".to_owned(),
    })
}

/// Drive an `Encryptor` to completion over an ASCII-armored in-memory
/// buffer, returning the finished bytes (ending with a newline).
fn seal(plaintext: &[u8], encryptor: Encryptor) -> Result<Vec<u8>, Error> {
    let mut output = Vec::new();
    let armored_writer =
        ArmoredWriter::wrap_output(&mut output, Format::AsciiArmor).map_err(Error::Io)?;
    let mut stream_writer = encryptor.wrap_output(armored_writer).map_err(Error::Io)?;
    stream_writer.write_all(plaintext).map_err(Error::Io)?;
    let armored_writer = stream_writer.finish().map_err(Error::Io)?;
    armored_writer.finish().map_err(Error::Io)?;
    if !output.ends_with(b"\n") {
        output.push(b'\n');
    }
    Ok(output)
}

/// Read the whole of `reader` into a zeroize-on-drop buffer, enforcing
/// [`MAX_PAYLOAD_BYTES`].
fn read_capped<R: Read>(reader: &mut R) -> Result<Zeroizing<Vec<u8>>, Error> {
    let mut buf = Zeroizing::new(Vec::new());
    let cap = u64::try_from(MAX_PAYLOAD_BYTES).unwrap_or(u64::MAX);
    reader
        .take(cap.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(Error::Io)?;
    if buf.len() > MAX_PAYLOAD_BYTES {
        return Err(Error::TooLarge {
            bytes: buf.len(),
            limit: MAX_PAYLOAD_BYTES,
        });
    }
    Ok(buf)
}

/// Map an `age` decrypt-phase error to the crate's error type. Called
/// only after a `Decryptor` was already constructed successfully, so
/// every case here means "the store could not be unlocked", never
/// "the store is not a valid age file".
fn map_decrypt_error(err: &DecryptError) -> Error {
    let reason = match err {
        DecryptError::NoMatchingKeys => "no matching identity",
        DecryptError::DecryptionFailed => "wrong passphrase or corrupted store",
        DecryptError::ExcessiveWork { .. } => "passphrase work factor too high",
        _ => "cannot unlock store",
    };
    Error::Unlock {
        reason: reason.to_owned(),
    }
}

/// Wrap an arbitrary `std::error::Error` as [`Error::Io`] without
/// leaking any secret material (`age`'s own error messages never
/// contain plaintext, passphrases, or identity material).
fn io_error(err: &(dyn std::error::Error + Send + Sync + 'static)) -> Error {
    Error::Io(std::io::Error::other(err.to_string()))
}
