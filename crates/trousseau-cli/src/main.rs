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
mod document;
mod exit;
mod help;
mod output;
mod prompt;

fn main() {
    // Rust ignores SIGPIPE at startup, turning a closed-pipe write into
    // a recoverable `io::Error` rather than killing the process. That
    // breaks `clap_complete`/`clap_mangen`, which `.expect()` on that
    // error internally (e.g. `completions fish | head` panics into a
    // panic report). Restoring the default disposition makes a broken
    // pipe kill the process via signal, quietly and non-zero, like any
    // other Unix tool, before that error can reach a `.expect()`.
    sigpipe::reset();

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

/// Install a panic hook that reports the panic message, location, this
/// build's version, and the operating system, and writes that same
/// information to a report file under the cache directory. Never
/// includes environment variables or command-line arguments, so a
/// secret passed on the command line or through `TROUSSEAU_PASSPHRASE`
/// cannot leak into either the report file or stderr (checked by the
/// `__panic-test` integration test in `tests/panic_test.rs`, which runs
/// against a debug build).
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let message = panic_message(info);
        let location = info
            .location()
            .map_or_else(|| "<unknown>".to_owned(), ToString::to_string);
        let report = format!(
            "trousseau {}\nos: {}\nlocation: {location}\nmessage: {message}\n",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
        );
        let file_path = write_panic_report(&report);
        crate::output::warn(&format!("error: trousseau crashed: {message}"));
        if let Some(path) = &file_path {
            crate::output::warn(&format!("a report was saved to {}", path.display()));
        }
        crate::output::warn(&format!(
            "this is a bug; please report it at {}/issues",
            env!("CARGO_PKG_REPOSITORY")
        ));
    }));
}

/// The panic payload as a string, the way the standard panic hook
/// prints it: a `&str` or `String` payload verbatim, anything else as a
/// fixed placeholder.
fn panic_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "Box<dyn Any>".to_owned())
}

/// Best-effort: write `report` under
/// `<cache_dir>/trousseau/panic-report-<pid>-<nanos>.txt`, returning the
/// path on success. Never panics: this runs from inside a panic hook,
/// where a second panic would abort the process.
fn write_panic_report(report: &str) -> Option<PathBuf> {
    let strategy = etcetera::choose_base_strategy().ok()?;
    let dir = {
        use etcetera::BaseStrategy as _;
        strategy.cache_dir().join("trousseau")
    };
    std::fs::create_dir_all(&dir).ok()?;
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let path = dir.join(format!("panic-report-{}-{unique}.txt", std::process::id()));
    std::fs::write(&path, report).ok()?;
    Some(path)
}
