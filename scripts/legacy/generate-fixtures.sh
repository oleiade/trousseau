#!/usr/bin/env bash
#
# generate-fixtures.sh
#
# Regenerates the legacy trousseau v0.4 store fixtures under
# crates/trousseau/tests/fixtures/legacy/, using the real Go `trousseau`
# binary built from the `go-final` tag (the last commit of the Go
# implementation, see docs/IMPLEMENTATION_PLAN.md step 0.1). The Rust
# `legacy` reader is tested against these fixtures, so they must be
# produced by the real legacy code, not reimplemented by hand.
#
# Requires `go`, `gpg` and `jq` on PATH. By default the script clones
# https://github.com/oleiade/trousseau.git at tag `go-final`; set
# TROUSSEAU_LEGACY_SRC to a local clone's path to clone from there
# instead (faster, no network needed), e.g.:
#
#   TROUSSEAU_LEGACY_SRC=/path/to/trousseau ./scripts/legacy/generate-fixtures.sh
#
# See crates/trousseau/tests/fixtures/legacy/README.md for what each
# fixture contains.

set -euo pipefail

PASSPHRASE='correct horse battery staple'
GPG_NAME='Trousseau Test'
GPG_EMAIL='test@trousseau.invalid'

# --- prerequisites ---------------------------------------------------------

for tool in go gpg git jq; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: '$tool' is required on PATH" >&2
    exit 1
  fi
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
fixture_dir="$repo_root/crates/trousseau/tests/fixtures/legacy"
mkdir -p "$fixture_dir"

tmp="$(mktemp -d)"
cleanup() { rm -rf "$tmp"; }
trap cleanup EXIT

# --- build the legacy Go binary --------------------------------------------

src="${TROUSSEAU_LEGACY_SRC:-https://github.com/oleiade/trousseau.git}"
echo "cloning $src (tag go-final)..." >&2
git clone --quiet --branch go-final --depth 1 "$src" "$tmp/src"

echo "building trousseau from go-final..." >&2
( cd "$tmp/src" && go build -o "$tmp/trousseau" ./cmd/trousseau )
bin="$tmp/trousseau"

# --- symmetric (AES-256-CFB) fixture ---------------------------------------

echo "generating symmetric fixture..." >&2

sym_dir="$tmp/symmetric"
mkdir -p "$sym_dir"
export TROUSSEAU_PASSPHRASE="$PASSPHRASE"

"$bin" --config "$sym_dir/config.toml" --store "$sym_dir/store.json" \
  create --encryption-type symmetric

"$bin" --config "$sym_dir/config.toml" --store "$sym_dir/store.json" \
  set abc 123
"$bin" --config "$sym_dir/config.toml" --store "$sym_dir/store.json" \
  set 'easy as' 'do re mi'

multiline_file="$tmp/multiline.txt"
printf 'a\nb' > "$multiline_file"
"$bin" --config "$sym_dir/config.toml" --store "$sym_dir/store.json" \
  set 'multi/line' --file "$multiline_file"

"$bin" --config "$sym_dir/config.toml" --store "$sym_dir/store.json" \
  set unicode 'héllo wörld'

cp "$sym_dir/store.json" "$fixture_dir/symmetric-v0.4.json"

# The decrypted inner document, reused below for expected.json and for the
# asymmetric fixture's fallback construction (both fixtures hold the same
# data, only the envelope differs).
"$bin" --config "$sym_dir/config.toml" --store "$sym_dir/store.json" \
  export --plain > "$tmp/inner.json"

unset TROUSSEAU_PASSPHRASE

# --- expected.json -----------------------------------------------------

jq -S '.data' "$tmp/inner.json" > "$fixture_dir/expected.json"

# --- asymmetric (OpenPGP) fixture ------------------------------------------

echo "generating throwaway OpenPGP key..." >&2

gnupg_home="$tmp/gnupg"
mkdir -p "$gnupg_home"
chmod 700 "$gnupg_home"

cat > "$tmp/gpg-key-params" <<EOF
%echo generating throwaway test key
Key-Type: RSA
Key-Length: 2048
Subkey-Type: RSA
Subkey-Length: 2048
Name-Real: $GPG_NAME
Name-Email: $GPG_EMAIL
Expire-Date: 0
Preferences: SHA256 SHA1 AES256 AES ZLIB ZIP Uncompressed
%no-protection
%commit
%echo done
EOF

GNUPGHOME="$gnupg_home" gpg --batch --gen-key "$tmp/gpg-key-params"

GNUPGHOME="$gnupg_home" gpg --batch --armor --export-secret-keys "$GPG_EMAIL" \
  > "$fixture_dir/test-key.sec.asc"
GNUPGHOME="$gnupg_home" gpg --batch --armor --export "$GPG_EMAIL" \
  > "$fixture_dir/test-key.pub.asc"

# The legacy Go binary reads keyring files directly (--gnupg-home DIR
# expects DIR/pubring.gpg and DIR/secring.gpg), not gpg's own keybox.
GNUPGHOME="$gnupg_home" gpg --export > "$gnupg_home/pubring.gpg"
GNUPGHOME="$gnupg_home" gpg --export-secret-keys > "$gnupg_home/secring.gpg"

asym_dir="$tmp/asymmetric"
mkdir -p "$asym_dir"

echo "attempting asymmetric fixture via the Go binary..." >&2

asym_method="go-binary"
# The passphrase env var is irrelevant to OpenPGP stores (the passphrase
# is only used for AES-256 stores), but the Go binary's passphrase
# resolution errors out on some platforms if it isn't set at all.
export TROUSSEAU_PASSPHRASE='unused-for-openpgp-stores'
if "$bin" --config "$asym_dir/config.toml" --gnupg-home "$gnupg_home" \
       --store "$asym_dir/store.json" create "$GPG_EMAIL" \
   && "$bin" --config "$asym_dir/config.toml" --gnupg-home "$gnupg_home" \
       --store "$asym_dir/store.json" set abc 123 \
   && "$bin" --config "$asym_dir/config.toml" --gnupg-home "$gnupg_home" \
       --store "$asym_dir/store.json" set 'easy as' 'do re mi' \
   && "$bin" --config "$asym_dir/config.toml" --gnupg-home "$gnupg_home" \
       --store "$asym_dir/store.json" set 'multi/line' --file "$multiline_file" \
   && "$bin" --config "$asym_dir/config.toml" --gnupg-home "$gnupg_home" \
       --store "$asym_dir/store.json" set unicode 'héllo wörld'
then
  cp "$asym_dir/store.json" "$fixture_dir/asymmetric-v0.4.json"
else
  echo "warning: the Go binary's deprecated OpenPGP library rejected the" >&2
  echo "  generated key (this is expected with modern gpg key preferences);" >&2
  echo "  falling back to hand-constructing the fixture." >&2
  asym_method="hand-constructed"

  GNUPGHOME="$gnupg_home" gpg --batch --yes --armor --trust-model always \
    --encrypt -r "$GPG_EMAIL" -o "$tmp/inner.json.asc" "$tmp/inner.json"

  data_b64="$(base64 < "$tmp/inner.json.asc" | tr -d '\n')"
  jq -nc --arg data "$data_b64" \
    '{crypto_type: 1, crypto_algorithm: 0, _data: $data}' \
    > "$fixture_dir/asymmetric-v0.4.json"

  # Verify the hand-constructed fixture actually decrypts to the same data
  # the symmetric fixture holds, using only the isolated GNUPGHOME.
  verify_home="$tmp/gnupg-verify"
  mkdir -p "$verify_home"
  chmod 700 "$verify_home"
  GNUPGHOME="$verify_home" gpg --batch --import "$fixture_dir/test-key.sec.asc" >/dev/null 2>&1

  jq -r '._data' "$fixture_dir/asymmetric-v0.4.json" | base64 -d > "$tmp/verify.asc"
  GNUPGHOME="$verify_home" gpg --batch --decrypt "$tmp/verify.asc" 2>/dev/null \
    | jq -S '.data' > "$tmp/verify-data.json"

  if ! diff -q "$tmp/verify-data.json" "$fixture_dir/expected.json" >/dev/null; then
    echo "error: hand-constructed asymmetric fixture does not decrypt to expected.json" >&2
    exit 1
  fi
  echo "verified: hand-constructed asymmetric fixture decrypts to expected.json" >&2
fi
unset TROUSSEAU_PASSPHRASE

# --- README ------------------------------------------------------------

cat > "$fixture_dir/README.md" <<EOF
# Legacy v0.4 fixtures

These fixtures are real trousseau v0.4 store files, produced by the Go
implementation at tag \`go-final\` (see \`docs/IMPLEMENTATION_PLAN.md\`
step 0.1). The Rust legacy reader (\`crates/trousseau/src/legacy.rs\`,
step 2.5) is tested against them. Do not hand-edit these files other
than the README; regenerate them with
\`scripts/legacy/generate-fixtures.sh\` instead.

## Files

- \`symmetric-v0.4.json\`: an AES-256-CFB (\`crypto_algorithm: 1\`) store,
  produced directly by the Go binary
  (\`create --encryption-type symmetric\`). Passphrase:
  \`$PASSPHRASE\`.

- \`asymmetric-v0.4.json\`: an OpenPGP (\`crypto_algorithm: 0\`) store,
  encrypted to the throwaway key below. Produced by: **$asym_method**.
$(if [ "$asym_method" = "go-binary" ]; then
cat <<'GOBIN'
  The Go binary's `create` and `set` commands ran successfully against
  the throwaway key with `--gnupg-home` pointed at an isolated GNUPGHOME.
GOBIN
else
cat <<'HANDCONSTRUCTED'
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
HANDCONSTRUCTED
fi)

- \`test-key.sec.asc\`, \`test-key.pub.asc\`: a throwaway, passphrase-less
  2048-bit RSA key pair (\`$GPG_NAME <$GPG_EMAIL>\`), generated solely to
  produce \`asymmetric-v0.4.json\`. It protects nothing real; do not
  treat it as sensitive, and do not reuse it for anything.

- \`expected.json\`: the plaintext \`data\` map both fixtures decrypt to
  (pretty-printed, keys sorted). Both fixtures were seeded with the same
  four entries: a plain value (\`abc\`), a key containing a space
  (\`easy as\`), a value with an embedded newline and a key containing a
  \`/\` (\`multi/line\`), and non-ASCII text (\`unicode\`).

## Regenerating

\`\`\`
./scripts/legacy/generate-fixtures.sh
\`\`\`

Requires \`go\`, \`gpg\` and \`jq\` on \`PATH\`. By default the script
clones \`https://github.com/oleiade/trousseau.git\` at tag \`go-final\`
over the network; set \`TROUSSEAU_LEGACY_SRC\` to a local clone's path
to clone from there instead:

\`\`\`
TROUSSEAU_LEGACY_SRC=/path/to/local/trousseau ./scripts/legacy/generate-fixtures.sh
\`\`\`

To check \`symmetric-v0.4.json\` by hand with the Go binary:

\`\`\`
TROUSSEAU_PASSPHRASE='$PASSPHRASE' trousseau \\
  --store crates/trousseau/tests/fixtures/legacy/symmetric-v0.4.json \\
  export --plain
\`\`\`

The \`data\` object in the output must match \`expected.json\`.
EOF

echo "wrote fixtures to $fixture_dir:" >&2
echo "  $fixture_dir/symmetric-v0.4.json" >&2
echo "  $fixture_dir/asymmetric-v0.4.json (method: $asym_method)" >&2
echo "  $fixture_dir/test-key.sec.asc" >&2
echo "  $fixture_dir/test-key.pub.asc" >&2
echo "  $fixture_dir/expected.json" >&2
echo "  $fixture_dir/README.md" >&2

exit 0
