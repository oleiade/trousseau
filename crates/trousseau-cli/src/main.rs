//! Entry point for the `trousseau` binary.
//!
//! Parses the command line, installs a panic hook that never leaks
//! secrets, builds the run [`context::Context`], dispatches to the
//! selected subcommand, and maps the result to a process exit code
//! through `exit.rs`.

use std::path::PathBuf;

use clap::Parser as _;

mod cli;
mod commands;
mod config;
mod context;
mod exit;
mod output;
mod prompt;

fn main() {
    install_panic_hook();

    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        // Clap usage errors keep clap's own exit code (2) and message
        // (3.5.1): `Error::exit` prints and exits without going through
        // `exit.rs`.
        Err(err) => err.exit(),
    };

    let ctx = match context::Context::new(&cli) {
        Ok(ctx) => ctx,
        Err(err) => {
            let mode = json_mode(cli.global.json);
            exit::report(&err, mode);
            std::process::exit(exit::code_for(&err));
        }
    };

    if let Err(err) = commands::dispatch(&cli, &ctx) {
        exit::report(&err, ctx.output);
        std::process::exit(exit::code_for(&err));
    }
}

/// The output mode to report an error in before a full
/// [`context::Context`] exists (building one can itself fail, for
/// example on a malformed configuration file).
const fn json_mode(json: bool) -> output::OutputMode {
    if json {
        output::OutputMode::Json
    } else {
        output::OutputMode::Human
    }
}

/// Install a panic hook that reports the panic message, this build's
/// version, and the operating system, and writes that same information
/// to a report file under the cache directory.
///
/// The report is built entirely from [`human_panic::report::Report`],
/// which never includes environment variables or command-line
/// arguments (checked by reading its fields: crate name and version,
/// operating system, panic location and message, and a backtrace of
/// function symbol names, none of which carry argv or env content).
/// This crate does not use `human_panic::setup_panic!`, because that
/// macro only installs its hook in release builds (`PanicStyle::Debug`
/// otherwise) and always writes its report to the system temp
/// directory; a custom hook, built from the same public `Report` and
/// `print_msg` API, is used instead so the report lands under this
/// build's own cache directory and the behavior is the same in debug
/// and release builds alike (the `__panic-test` integration test in
/// `tests/panic_test.rs` runs against a debug build).
fn install_panic_hook() {
    let metadata = human_panic::Metadata::new("trousseau", env!("CARGO_PKG_VERSION"))
        .authors(env!("CARGO_PKG_AUTHORS").replace(':', ", "))
        .repository(env!("CARGO_PKG_REPOSITORY"));
    std::panic::set_hook(Box::new(move |info| {
        let report = human_panic::report::Report::with_panic(&metadata, info);
        let file_path = write_panic_report(&report);
        let _ = human_panic::print_msg(file_path.as_deref(), &metadata);
    }));
}

/// Best-effort: serialize `report` and write it under
/// `<cache_dir>/trousseau/panic-report-<pid>-<nanos>.toml`, returning
/// the path on success. Never panics: this runs from inside a panic
/// hook, where a second panic would abort the process.
fn write_panic_report(report: &human_panic::report::Report) -> Option<PathBuf> {
    let toml = report.serialize()?;
    let strategy = etcetera::choose_base_strategy().ok()?;
    let dir = {
        use etcetera::BaseStrategy as _;
        strategy.cache_dir().join("trousseau")
    };
    std::fs::create_dir_all(&dir).ok()?;
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let path = dir.join(format!("panic-report-{}-{unique}.toml", std::process::id()));
    std::fs::write(&path, toml).ok()?;
    Some(path)
}
