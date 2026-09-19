//! Golden-fixture and property-based round-trip tests for the schema 1
//! payload format. See `docs/IMPLEMENTATION_PLAN.md` section 3.1 and
//! `docs/format.md`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use proptest::prelude::*;
use time::macros::datetime;
use trousseau::schema::{Key, Store, StoreKind, Value};

/// The example payload from `docs/IMPLEMENTATION_PLAN.md` section 3.1.2
/// and `docs/format.md`, generated with [`Store::to_json`] (see the step
/// 2.1 PR description for how it was produced) and checked by eye
/// against the plan.
const GOLDEN_FIXTURE: &str = include_str!("fixtures/schema1.json");

#[test]
fn golden_fixture_round_trips_byte_identically() {
    let store = Store::from_json(GOLDEN_FIXTURE.as_bytes()).expect("fixture parses and validates");
    let reserialized = store.to_json().expect("a valid store serializes");
    assert_eq!(
        String::from_utf8(reserialized).expect("to_json produces utf8"),
        GOLDEN_FIXTURE,
        "re-serializing the golden fixture must be byte-identical to the fixture"
    );
}

/// One `/`-separated key segment: `[A-Za-z0-9][A-Za-z0-9._-]{0,12}`.
fn key_segment() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zA-Z0-9][a-zA-Z0-9._-]{0,12}").expect("valid regex")
}

/// A key with one to three segments.
fn arb_key() -> impl Strategy<Value = Key> {
    proptest::collection::vec(key_segment(), 1..=3)
        .prop_map(|segments| Key::parse(&segments.join("/")).expect("generated key is valid"))
}

/// An optional explicit `env` override, matching `^[A-Za-z_][A-Za-z0-9_]*$`.
fn arb_env() -> impl Strategy<Value = Option<String>> {
    proptest::option::of(
        proptest::string::string_regex("[A-Za-z_][A-Za-z0-9_]{0,10}").expect("valid regex"),
    )
}

/// An optional description, well within the 1024-byte limit.
fn arb_description() -> impl Strategy<Value = Option<String>> {
    proptest::option::of("[ -~]{0,32}")
}

/// Arbitrary value bytes: [`Value::detect_encoding`] decides `utf8` vs
/// `base64` for each generated value.
fn arb_value_bytes() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..32)
}

proptest! {
    /// `Store::from_json(store.to_json()) == store` for stores built from
    /// randomly generated valid keys, arbitrary (utf8 or binary) values,
    /// and optional `env` / `description` metadata.
    #[test]
    fn store_round_trips_through_json(
        entries in proptest::collection::vec(
            (arb_key(), arb_value_bytes(), arb_env(), arb_description()),
            0..8,
        ),
    ) {
        let now = datetime!(2026-09-12 09:41:00 UTC);
        let mut store = Store::new(StoreKind::Passphrase, Vec::new(), now);
        for (key, bytes, env, description) in entries {
            let value = Value::from_bytes(bytes).expect("value within the size limit");
            store.set(key, value, env, description, now);
        }

        let json = store.to_json().expect("a store built through the public API validates");
        let parsed = Store::from_json(&json).expect("a store's own to_json output parses");
        prop_assert_eq!(parsed, store);
    }
}
