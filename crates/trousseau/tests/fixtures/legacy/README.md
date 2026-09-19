# Legacy v0.4 fixtures

These fixtures are real trousseau v0.4 store files, produced by the Go
implementation at tag `go-final` (see `docs/IMPLEMENTATION_PLAN.md`
step 0.1). The Rust legacy reader (`crates/trousseau/src/legacy.rs`,
step 2.5) is tested against them. Do not hand-edit these files other
than the README; regenerate them with
`scripts/legacy/generate-fixtures.sh` instead.

## Files

- `symmetric-v0.4.json`: an AES-256-CFB (`crypto_algorithm: 1`) store,
  produced directly by the Go binary
  (`create --encryption-type symmetric`). Passphrase:
  `correct horse battery staple`.

- `asymmetric-v0.4.json`: an OpenPGP (`crypto_algorithm: 0`) store,
  encrypted to the throwaway key below. Produced by: **hand-constructed**.
  The Go binary's `create` command failed against this key
  (`No common hashes for encryption keys` — the deprecated
  `golang.org/x/crypto/openpgp`-derived library the Go implementation
  uses does not support the hash algorithm preferences modern `gpg`
  writes into a newly generated key). The fixture was instead
  constructed by hand, byte-for-byte what the Go code would produce:
  the symmetric fixture's inner plaintext document was decrypted with
  the Go binary (`export --plain`), re-encrypted with
  `gpg --armor --encrypt -r test@trousseau.invalid --trust-model always`,
  base64-encoded, and wrapped as
  `{"crypto_type":1,"crypto_algorithm":0,"_data":"<base64>"}` (the exact
  field set and order `encoding/json` produces for the Go `Vault`
  struct). The regeneration script verifies this fixture by
  base64-decoding `_data`, decrypting it with `gpg --batch --decrypt`
  under an isolated `GNUPGHOME` seeded only from `test-key.sec.asc`, and
  diffing the resulting `data` object against `expected.json`.

- `test-key.sec.asc`, `test-key.pub.asc`: a throwaway, passphrase-less
  2048-bit RSA key pair (`Trousseau Test <test@trousseau.invalid>`), generated solely to
  produce `asymmetric-v0.4.json`. It protects nothing real; do not
  treat it as sensitive, and do not reuse it for anything.

- `expected.json`: the plaintext `data` map both fixtures decrypt to
  (pretty-printed, keys sorted). Both fixtures were seeded with the same
  four entries: a plain value (`abc`), a key containing a space
  (`easy as`), a value with an embedded newline and a key containing a
  `/` (`multi/line`), and non-ASCII text (`unicode`).

## Regenerating

```
./scripts/legacy/generate-fixtures.sh
```

Requires `go`, `gpg` and `jq` on `PATH`. By default the script
clones `https://github.com/oleiade/trousseau.git` at tag `go-final`
over the network; set `TROUSSEAU_LEGACY_SRC` to a local clone's path
to clone from there instead:

```
TROUSSEAU_LEGACY_SRC=/path/to/local/trousseau ./scripts/legacy/generate-fixtures.sh
```

To check `symmetric-v0.4.json` by hand with the Go binary:

```
TROUSSEAU_PASSPHRASE='correct horse battery staple' trousseau \
  --store crates/trousseau/tests/fixtures/legacy/symmetric-v0.4.json \
  export --plain
```

The `data` object in the output must match `expected.json`.
