# Trousseau

Trousseau is a portable, age-encrypted keyring for storing and sharing secrets from the command line.

[![CI](https://github.com/oleiade/trousseau/actions/workflows/ci.yml/badge.svg)](https://github.com/oleiade/trousseau/actions/workflows/ci.yml)

## Quick start

```bash
trousseau init                                 # create a store, generate an identity
trousseau set database/password                # hidden prompt, or --from-file
trousseau run -- psql "$DATABASE_URL"           # inject secrets, run your program

git add .trousseau
git commit -m "add encrypted secrets store"

# a teammate sends you their age or SSH recipient, you add it
trousseau recipients add age1qz3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
git commit -am "add teammate as a recipient"
```

## What it defends against

- Disclosure of the store at rest: a stolen laptop, a leaked bucket, a public repository, a synced folder.
- Undetected modification of the store by a non-recipient.
- Secrets in shell history, process lists, and crash logs via argv. Trousseau never accepts a secret value as a command-line argument.

## What it does not defend against

- A compromised host while an identity usable on that host exists.
- A malicious recipient rewriting the store. There are no signatures in v1; git history is the audit trail.
- Clipboard managers that keep history, and editors that leave backup or swap files during `edit`.

See `docs/threat-model.md` for the full list and the reasoning behind it.

## Install

### Homebrew (macOS and Linux)

```bash
brew tap oleiade/tap
brew install trousseau
```

### apt (Debian and Ubuntu)

```bash
curl -fsSL https://oleiade.github.io/deb/oleiade-archive-keyring.gpg | \
  gpg --dearmor | \
  sudo tee /usr/share/keyrings/oleiade-archive-keyring.gpg > /dev/null
echo "deb [signed-by=/usr/share/keyrings/oleiade-archive-keyring.gpg] https://oleiade.github.io/deb stable main" | \
  sudo tee /etc/apt/sources.list.d/oleiade.list > /dev/null
sudo apt update
sudo apt install trousseau
```

### GitHub release archives

Download a `tar.gz` from the [releases page](https://github.com/oleiade/trousseau/releases) and verify it before trusting it. Every asset is attested and checksummed; see `docs/cli.md`'s "Verify a download" section for the exact commands. Prebuilt Windows binaries are not published yet.

### `cargo install`

Trousseau is not yet published to crates.io. Install straight from the repository:

```bash
cargo install --git https://github.com/oleiade/trousseau trousseau-cli
```

This builds the `trousseau` binary and installs it to `~/.cargo/bin`, on Windows included.

## The git workflow

A project store is a single file, `.trousseau`, committed next to your code:

```bash
git add .trousseau
git commit -m "add encrypted secrets store"
```

Add a teammate by their age or SSH recipient. They send you a public value, never a secret:

```bash
trousseau recipients add age1qz3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
git commit -am "add teammate as a recipient"
```

Rotate the store after removing someone, or after any suspected leak of the file:

```bash
trousseau rekey
git commit -am "rekey after removing a recipient"
```

Every save re-encrypts the whole store under a fresh key, so `rekey` and a normal `recipients rm` behave the same way on disk.

## CI usage

Write an identity from a CI secret to a file, then point trousseau at it:

```bash
echo "$CI_TROUSSEAU_IDENTITY" > /tmp/identity.txt
export TROUSSEAU_IDENTITY_FILE=/tmp/identity.txt
eval "$(trousseau env)"
```

`trousseau run -- your-build-step` works the same way, without the `eval`. Nothing here puts a secret value on the command line or in a log.

## Migrating from v0.4

If you have a `~/.trousseau` file from the old Go binary:

```bash
trousseau migrate ~/.trousseau
trousseau ls --long
```

See `docs/migration.md` for passphrase and OpenPGP stores, and for what happens to key names that do not fit the new grammar.

## Documentation

- [`docs/format.md`](docs/format.md): the on-disk envelope, payload schema, and key grammar.
- [`docs/cli.md`](docs/cli.md): every command, flag, exit code, and JSON shape.
- [`docs/threat-model.md`](docs/threat-model.md): what trousseau defends against, in full.
- [`docs/migration.md`](docs/migration.md): moving from the v0.4 Go binary.
- [`CONTRIBUTING.md`](CONTRIBUTING.md): toolchain, checks, and how to add a command.

The Go implementation lives on the `legacy-go` branch and the `go-final` tag; its last release is v0.4.1.

## License

MIT. See [`LICENSE`](LICENSE).
