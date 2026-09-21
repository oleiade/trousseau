# Run the full local quality gate: format check, lint, tests, and docs.
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Run the workspace test suite.
test:
    cargo test --workspace --all-features

# Run clippy across every target and feature, denying warnings.
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format the workspace in place.
fmt:
    cargo fmt --all

# Build a native release binary and use it to generate shell completions
# and the man page, for the release archives (step 4.1). Run from a
# GoReleaser `before` hook. These land under target/release-artifacts,
# not dist/: GoReleaser requires dist/ to still be empty after `before`
# hooks run (even under --clean), so anything the hook writes must live
# elsewhere.
generate-artifacts:
    cargo build --release -p trousseau-cli
    mkdir -p target/release-artifacts/completions target/release-artifacts/man
    ./target/release/trousseau completions bash > target/release-artifacts/completions/trousseau.bash
    ./target/release/trousseau completions zsh > target/release-artifacts/completions/trousseau.zsh
    ./target/release/trousseau completions fish > target/release-artifacts/completions/trousseau.fish
    ./target/release/trousseau man > target/release-artifacts/man/trousseau.1

# Build a snapshot release locally without publishing, to sanity-check
# the GoReleaser configuration (step 4.1).
release-dry-run:
    goreleaser release --snapshot --clean

# Generate a CycloneDX SBOM for the released binary (step 4.2). Requires
# cargo-cyclonedx on PATH: `cargo install cargo-cyclonedx --locked`. Runs
# across the whole workspace (cargo-cyclonedx writes one file per crate,
# next to each Cargo.toml, regardless of --manifest-path); the CLI
# crate's file is what ships as the release asset, so it's moved to the
# repo root and renamed, and the library crate's file is discarded.
sbom:
    cargo cyclonedx --format json --all
    mv crates/trousseau-cli/trousseau-cli.cdx.json trousseau.cdx.json
    rm -f crates/trousseau/trousseau.cdx.json
