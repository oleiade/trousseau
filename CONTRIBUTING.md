# Contributing

Trousseau is a Cargo workspace: a library crate (`crates/trousseau`) and a thin CLI on top of it (`crates/trousseau-cli`). This document covers the local toolchain, the checks a change must pass, and how a PR is expected to look.

## Toolchain

`rust-toolchain.toml` pins the toolchain for you; `rustup` picks it up automatically in this directory. The workspace's minimum supported Rust version is **1.89**, set in the root `Cargo.toml`'s `rust-version`. CI checks both the pinned stable toolchain and a build against 1.89 directly, so do not rely on an API stabilized after 1.89.

Install the components the toolchain file asks for (`rustfmt`, `clippy`) and `just`:

```bash
rustup component add rustfmt clippy
cargo install just
```

## Running the checks

```bash
just check
```

This runs, in order: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, and `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"`. Run it before opening a PR. `just fmt` formats in place, `just test` and `just lint` run one step at a time.

`cargo deny check` is not part of `just check` but is part of CI; run it directly if you touched dependencies.

## Lint policy

Both crates deny `clippy::pedantic`, `clippy::nursery`, `clippy::unwrap_used`, `clippy::expect_used`, and `clippy::panic`, and forbid `unsafe_code`. `unwrap()`, `expect()`, and `panic!()` are allowed in test modules only. The library crate also denies `clippy::print_stdout` and `clippy::print_stderr`: it has no business writing to either. The CLI crate warns on the same two lints at the crate level and allows them only inside `src/output.rs`, which every other module goes through for anything user-facing.

If a lint genuinely does not apply to a specific line, scope the `#[allow]` as tightly as possible and say why in a comment. Do not weaken the crate-level lint configuration to make a change pass.

## PR expectations

One step of `docs/IMPLEMENTATION_PLAN.md` per PR while the plan is in progress; afterward, one coherent change per PR. Do not bundle unrelated fixes into a feature PR, and do not start the next piece of work in a branch that still has an open PR.

Use the PR template (`.github/PULL_REQUEST_TEMPLATE.md`): what the PR does, why, how to review it, the test evidence (paste the summary lines, plus any manual command you ran), deviations from the plan with reasons, and follow-ups you noticed but left out of scope.

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat(cli): add get command`, `fix(lib): reject empty keys`, `docs: update cli.md`, `test: cover env name conflicts`.

A PR is ready for review once `just check` and `cargo deny check` pass locally, and CI is green.

## How to add a command

A new subcommand touches the same four places every time:

1. **`crates/trousseau-cli/src/cli.rs`**: add the variant to the `Command` enum and its `Args` struct, with doc comments (they become `--help` text).
2. **`crates/trousseau-cli/src/commands/`**: add a module with a `run` function taking the resolved `Context` and the parsed args, and wire it into `dispatch` in `commands/mod.rs`.
3. **`docs/cli.md`**: document the command's usage line, its behavior, exit codes it can return beyond the general table, and its `--json` shape if it has one. `docs/cli.md` restates `docs/IMPLEMENTATION_PLAN.md` section 3.5 and must not contradict it; if the plan does not cover the command yet, propose the plan change first.
4. **Tests**: an integration test in `crates/trousseau-cli/tests/`, in the style of the existing command test files (see `tests/run_env.rs` for a command with several flags, `tests/common/mod.rs` for the shared `Env` test harness). Cover the happy path, the JSON output if any, and every exit code the command can return.

Secret values follow the same rules as every other command: never on argv, never in a log line, never in `Debug` output, never in a file outside what the specification names.
