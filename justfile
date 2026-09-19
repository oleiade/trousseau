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
