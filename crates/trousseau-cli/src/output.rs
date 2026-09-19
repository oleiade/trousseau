//! Human and JSON output.
//!
//! This is the only module in the crate allowed to print: every
//! user-facing write to stdout or stderr goes through one of these
//! functions. That is why the crate-level `print_stdout`/`print_stderr`
//! lints are set to `warn` and `#[allow]`ed only in this file (see
//! `src/main.rs`'s crate attributes).
//!
//! [`table`] has no caller yet: `ls --long` starts using it in step 3.3b.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::Write as _;

use serde::Serialize;

/// Whether output goes to a human or to a JSON consumer (3.5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Plain text, formatted for a person reading a terminal.
    Human,
    /// Exactly one JSON document on stdout for a read command, or
    /// `{"ok": true, ...}` for a write command. Errors go to stderr as a
    /// JSON object.
    Json,
}

/// Print `msg` to stderr, unless `quiet` suppresses informational lines
/// (3.5.1). Used for progress notes such as `set KEY` or `created
/// <path>`.
pub fn info(quiet: bool, msg: &str) {
    if quiet {
        return;
    }
    eprintln!("{msg}");
}

/// Print `msg` to stderr unconditionally. Used for warnings that matter
/// regardless of `--quiet`, such as "you removed your own recipient" or
/// a skipped binary entry.
pub fn warn(msg: &str) {
    eprintln!("{msg}");
}

/// Serialize `value` as one pretty JSON document, with a trailing
/// newline, to stdout (3.5.1).
///
/// # Errors
///
/// Returns an error if `value` cannot be serialized, or if stdout
/// cannot be written to.
pub fn json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(text.as_bytes())?;
    Ok(())
}

/// Print `error: <message>` to stderr (human mode error reporting; see
/// `crate::exit::report`).
pub fn error_human(message: &str) {
    eprintln!("error: {message}");
}

/// Print `{"error": {"code": "<code>", "message": "<message>"}}` to
/// stderr (JSON mode error reporting; see `crate::exit::report`).
pub fn error_json(code: &str, message: &str) {
    #[derive(Serialize)]
    struct ErrorBody<'a> {
        code: &'a str,
        message: &'a str,
    }
    #[derive(Serialize)]
    struct Envelope<'a> {
        error: ErrorBody<'a>,
    }
    let envelope = Envelope {
        error: ErrorBody { code, message },
    };
    if let Ok(text) = serde_json::to_string(&envelope) {
        eprintln!("{text}");
    }
}

/// Print `msg` and a trailing newline to stdout, unconditionally (not
/// suppressed by `--quiet`, unlike [`info`]): used for a command's own
/// primary human-readable result line, such as one of `info`'s aligned
/// label lines, as opposed to an incidental progress note.
pub fn line(msg: &str) {
    println!("{msg}");
}

/// Write `bytes` to stdout verbatim: no added newline, no encoding.
/// Used for `get`'s default output and similar raw-bytes commands.
///
/// # Errors
///
/// Returns an error if stdout cannot be written to.
pub fn raw(bytes: &[u8]) -> anyhow::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(bytes)?;
    Ok(())
}

/// Print a simple, dependency-free aligned table to stdout: `headers`
/// as the first row, then `rows`, each column padded to the widest
/// entry in that column (including the header).
#[allow(dead_code)]
pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(cell.len());
            }
        }
    }
    println!(
        "{}",
        format_row(headers.iter().map(|h| (*h).to_owned()), &widths)
    );
    for row in rows {
        println!("{}", format_row(row.iter().cloned(), &widths));
    }
}

/// Pad and join one table row per `widths`, trimming trailing
/// whitespace from the last column.
fn format_row(cells: impl Iterator<Item = String>, widths: &[usize]) -> String {
    let mut line = String::new();
    for (index, cell) in cells.enumerate() {
        let width = widths.get(index).copied().unwrap_or(cell.len());
        if index > 0 {
            line.push_str("  ");
        }
        line.push_str(&cell);
        for _ in cell.len()..width {
            line.push(' ');
        }
    }
    line.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::format_row;

    #[test]
    fn format_row_pads_and_trims() {
        // Column 0 is padded to width 4 ("a" + 3 spaces), then the
        // 2-space column separator, then column 1 ("bb") needs no
        // padding since it is already its column's full width, and any
        // trailing whitespace is trimmed.
        let widths = [4, 2];
        let line = format_row(["a".to_owned(), "bb".to_owned()].into_iter(), &widths);
        assert_eq!(line, "a     bb");
    }
}
