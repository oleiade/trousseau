//! Entry point for the `trousseau` binary.
//!
//! This step only wires up a bare clap `Command` that answers
//! `--version`. Subcommands, configuration, and output formatting are
//! added in later steps of `docs/IMPLEMENTATION_PLAN.md`.

use clap::Command;

fn main() {
    Command::new("trousseau")
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();
}
