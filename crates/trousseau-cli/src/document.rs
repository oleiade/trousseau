//! The TOML document format shared by `export --format toml`,
//! `import --format toml` (3.5.11), and `edit` (3.5.12, step 3.7).
//!
//! One TOML table per entry, keyed by the entry's key string:
//!
//! ```toml
//! # trousseau edit: /home/t/project/.trousseau
//! # Save and quit to apply. Delete a table to remove its entry.
//! # Leave the file empty to abort.
//!
//! ["database/password"]
//! value = "s3cr3t"
//! env = "DATABASE_PASSWORD"
//! description = "Postgres app role"
//!
//! ["tls/server.key"]
//! value_base64 = "LS0tLS1CRUdJTi4uLg=="
//! ```
//!
//! A table carries exactly one of `value` (a `utf8` entry, verbatim) or
//! `value_base64` (a `base64` entry, standard padded base64); `env` and
//! `description` are optional. [`to_toml`] and [`from_toml`] carry no
//! timestamps: a caller applying a parsed [`DocEntry`] decides
//! `created_at`/`updated_at` itself (`edit` keeps an entry's existing
//! timestamps or bumps them per 3.5.12; `import` always sets
//! `updated_at` to now and has no `created_at` to keep for this format,
//! per 3.5.11).

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};

use trousseau::schema::{Encoding, Key, Store, Value};

/// One parsed (or about-to-be-rendered) table from the document format:
/// an entry stripped of the timestamps [`Store`] carries alongside it.
#[derive(Debug)]
pub struct DocEntry {
    /// The entry's secret bytes.
    pub value: Value,
    /// Whether `value` came from (or renders to) `value` (utf8) or
    /// `value_base64` (base64).
    pub encoding: Encoding,
    /// An explicit environment variable name override (3.1.4).
    pub env: Option<String>,
    /// Free-text description.
    pub description: Option<String>,
}

/// One table's on-disk shape, used only to drive `toml`'s (de)serializer.
/// Kept private: callers use [`DocEntry`], never this type.
#[derive(Serialize)]
struct TomlEntryOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// The same shape, for parsing. `deny_unknown_fields` so a stray field
/// surfaces as a parse error rather than being silently dropped.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlEntryIn {
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    value_base64: Option<String>,
    #[serde(default)]
    env: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

/// Render one `NAME="value"` dotenv line, with a trailing newline,
/// escaping `\`, `"`, and a newline in `value` as `\\`, `\"`, `\n`
/// (backslash first, so the other escapes are not doubled). Shared by
/// `export --format dotenv` (3.5.11) and `env --format dotenv` (3.5.14).
#[must_use]
pub fn dotenv_line(name: &str, value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("{name}=\"{escaped}\"\n")
}

/// Render `store`'s entries as the TOML document format, prefixed by
/// `header` (the caller's own comment lines, without a trailing blank
/// line; pass an empty string for none).
///
/// Entries are rendered in key order (the same order [`Store::entries`],
/// a `BTreeMap`, already iterates in).
#[must_use]
pub fn to_toml(store: &Store, header: &str) -> String {
    let mut tables: BTreeMap<&str, TomlEntryOut> = BTreeMap::new();
    for (key, entry) in &store.entries {
        let (value, value_base64) = match entry.encoding {
            // Defensive only, in the `Err` case: a validated store
            // never has a `utf8` entry whose bytes are not valid UTF-8.
            // Fall back to base64 rather than losing the value.
            Encoding::Utf8 => std::str::from_utf8(entry.value.expose()).map_or_else(
                |_| (None, Some(STANDARD.encode(entry.value.expose()))),
                |text| (Some(text.to_owned()), None),
            ),
            Encoding::Base64 => (None, Some(STANDARD.encode(entry.value.expose()))),
        };
        tables.insert(
            key.as_str(),
            TomlEntryOut {
                value,
                value_base64,
                env: entry.env.clone(),
                description: entry.description.clone(),
            },
        );
    }
    // `toml::to_string_pretty` cannot fail for this shape (every field
    // is a plain string); an empty document on the (unreachable) error
    // path is preferable to a panic.
    let body = toml::to_string_pretty(&tables).unwrap_or_default();
    if header.is_empty() {
        body
    } else {
        let mut out = header.trim_end_matches('\n').to_owned();
        out.push_str("\n\n");
        out.push_str(&body);
        out
    }
}

/// Parse the TOML document format into one [`DocEntry`] per table,
/// keyed by the parsed, validated [`Key`].
///
/// # Errors
///
/// Returns an error if `text` is not well-formed TOML, if any top-level
/// key is not a valid [`Key`] (3.1.3), if a table does not have exactly
/// one of `value`/`value_base64`, or if `value_base64` is not valid
/// standard base64. Every error mentions the offending key.
pub fn from_toml(text: &str) -> anyhow::Result<BTreeMap<Key, DocEntry>> {
    let raw: BTreeMap<String, TomlEntryIn> =
        toml::from_str(text).map_err(|err| anyhow::anyhow!("parsing toml document: {err}"))?;

    let mut result = BTreeMap::new();
    for (key_str, table) in raw {
        let key =
            Key::parse(&key_str).map_err(|err| anyhow::anyhow!("{key_str}: invalid key: {err}"))?;
        let entry = doc_entry_from_table(&key_str, table)?;
        result.insert(key, entry);
    }
    Ok(result)
}

/// Build one [`DocEntry`] from a parsed table, enforcing "exactly one of
/// `value`/`value_base64`" (3.5.12).
fn doc_entry_from_table(key_str: &str, table: TomlEntryIn) -> anyhow::Result<DocEntry> {
    let (value, encoding) = match (table.value, table.value_base64) {
        (Some(text), None) => {
            let value = Value::from_bytes(text.into_bytes())
                .map_err(|err| anyhow::anyhow!("{key_str}: {err}"))?;
            (value, Encoding::Utf8)
        }
        (None, Some(encoded)) => {
            let bytes = STANDARD
                .decode(encoded.as_bytes())
                .map_err(|err| anyhow::anyhow!("{key_str}: invalid value_base64: {err}"))?;
            let value =
                Value::from_bytes(bytes).map_err(|err| anyhow::anyhow!("{key_str}: {err}"))?;
            (value, Encoding::Base64)
        }
        (Some(_), Some(_)) => anyhow::bail!(
            "{key_str}: table must have exactly one of `value` or `value_base64`, not both"
        ),
        (None, None) => {
            anyhow::bail!("{key_str}: table must have exactly one of `value` or `value_base64`")
        }
    };
    Ok(DocEntry {
        value,
        encoding,
        env: table.env,
        description: table.description,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{from_toml, to_toml};
    use time::OffsetDateTime;
    use trousseau::schema::{Store, StoreKind};

    #[test]
    fn round_trips_utf8_and_base64_entries() {
        let now = OffsetDateTime::now_utc();
        let mut store = Store::new(StoreKind::Recipients, vec!["age1x".to_owned()], now);
        store.set(
            "database/password".parse().unwrap(),
            trousseau::schema::Value::from_bytes(b"s3cr3t".to_vec()).unwrap(),
            Some("DATABASE_PASSWORD".to_owned()),
            Some("Postgres app role".to_owned()),
            now,
        );
        store.set(
            "tls/server.key".parse().unwrap(),
            trousseau::schema::Value::from_bytes(vec![0x00, 0x01, 0xff]).unwrap(),
            None,
            None,
            now,
        );

        let text = to_toml(&store, "# trousseau edit: /x/.trousseau");
        assert!(text.starts_with("# trousseau edit: /x/.trousseau\n\n"));

        let parsed = from_toml(&text).expect("parse");
        assert_eq!(parsed.len(), 2);

        let password = &parsed[&"database/password".parse().unwrap()];
        assert_eq!(password.value.expose(), b"s3cr3t");
        assert_eq!(password.encoding, trousseau::schema::Encoding::Utf8);
        assert_eq!(password.env.as_deref(), Some("DATABASE_PASSWORD"));
        assert_eq!(password.description.as_deref(), Some("Postgres app role"));

        let tls = &parsed[&"tls/server.key".parse().unwrap()];
        assert_eq!(tls.value.expose(), &[0x00, 0x01, 0xff]);
        assert_eq!(tls.encoding, trousseau::schema::Encoding::Base64);
        assert_eq!(tls.env, None);
    }

    #[test]
    fn empty_header_produces_no_leading_blank_line() {
        let now = OffsetDateTime::now_utc();
        let store = Store::new(StoreKind::Recipients, vec!["age1x".to_owned()], now);
        let text = to_toml(&store, "");
        assert!(!text.starts_with('\n'));
    }

    #[test]
    fn rejects_a_table_with_neither_value_field() {
        let err = from_toml("[\"k\"]\nenv = \"K\"\n").unwrap_err();
        assert!(err.to_string().contains('k'));
    }

    #[test]
    fn rejects_a_table_with_both_value_fields() {
        let err = from_toml("[\"k\"]\nvalue = \"a\"\nvalue_base64 = \"YQ==\"\n").unwrap_err();
        assert!(err.to_string().contains('k'));
    }

    #[test]
    fn rejects_an_invalid_key() {
        let err = from_toml("[\"\"]\nvalue = \"a\"\n").unwrap_err();
        assert!(err.to_string().contains("invalid key"));
    }
}
