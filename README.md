# Trousseau

Trousseau is a portable, encrypted keyring for storing and sharing secrets from the command line.

Rewrite in progress on the `rust-rewrite` branch; the Go implementation lives on the `legacy-go` branch and the `go-final` tag; the last Go release is v0.4.1.

See [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for the rewrite plan.

## Install

### Homebrew (macOS and Linux)

```bash
brew tap oleiade/tap
brew install trousseau
```

### apt (Debian and Ubuntu)

```bash
# Download and install the repository's GPG key
curl -fsSL https://oleiade.github.io/deb/oleiade-archive-keyring.gpg | \
  gpg --dearmor | \
  sudo tee /usr/share/keyrings/oleiade-archive-keyring.gpg > /dev/null

# Add the repository to your system's sources
echo "deb [signed-by=/usr/share/keyrings/oleiade-archive-keyring.gpg] https://oleiade.github.io/deb stable main" | \
  sudo tee /etc/apt/sources.list.d/oleiade.list > /dev/null

# Update your sources and install
sudo apt update
sudo apt install trousseau
```

### GitHub release archives

Download a `tar.gz` (or, on Windows, a `zip` once published — see below) from the
[releases page](https://github.com/oleiade/trousseau/releases), and verify it before
trusting it: see [`docs/cli.md`'s "Verify a download"](docs/cli.md#verify-a-download)
section for the `gh attestation verify` and `sha256sum -c` commands.

### `cargo install`

Trousseau is not yet published to crates.io. Install straight from the repository:

```bash
cargo install --git https://github.com/oleiade/trousseau trousseau-cli
```

This builds the `trousseau` binary from the `trousseau-cli` workspace member and
installs it to `~/.cargo/bin`.

### Windows

Prebuilt Windows binaries are not published yet; `cargo install --git` above works
on Windows today.
