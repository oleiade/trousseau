//! Long-form help: the examples and notes printed after each command's
//! options. Kept out of `cli.rs` so the argument definitions stay
//! readable. Plain text only; see `docs/HELP_TEXT_PLAN.md` section 4 for
//! the conventions and the test at the bottom of this file that keeps
//! every example parseable.

/// The top level's short `after_help`, shown by `-h`.
pub const TOP: &str = r"Quick start:
  trousseau init                    Create a store in this directory
  trousseau set database/password   Save a secret (prompts for the value)
  trousseau get database/password   Print it
  trousseau ls                      List the keys
  trousseau run -- ./server         Run a command with the secrets in its env

Run 'trousseau help <command>' for details and examples.
Run 'trousseau --help' for store selection, unlocking, and exit codes.";

/// The top level's long `after_long_help`, shown by `--help`.
pub const TOP_LONG: &str = r"Quick start:
  trousseau init                    Create a store in this directory
  trousseau set database/password   Save a secret (prompts for the value)
  trousseau get database/password   Print it
  trousseau ls                      List the keys
  trousseau run -- ./server         Run a command with the secrets in its env

Which store a command uses (first match wins):
  1. --store PATH
  2. the TROUSSEAU_STORE environment variable
  3. with --global: your personal store
  4. the nearest '.trousseau' file, from this directory upward
  5. your personal store
  'trousseau info' prints the store a command would use.
  'trousseau init' never walks up: it creates '.trousseau' right here, or
  your personal store with --global.

How a store is unlocked:
  A recipients store opens with any matching identity (private key) from
  --identity, TROUSSEAU_IDENTITY_FILE, 'identity.files' in the config file,
  the default identity file that 'init' creates, ~/.ssh/id_ed25519, and
  ~/.ssh/id_rsa.
  A passphrase store prompts for its passphrase, or reads --passphrase-file.

Exit codes:
  0  success                     5  key not found
  1  generic failure             6  store locked by another process
  2  usage error or refusal      7  legacy v0.4 store: run 'migrate'
  3  store not found             8  conflict (already exists, env clash)
  4  cannot unlock the store
  'run' exits with the child command's exit code.

Run 'trousseau help <command>' for details and examples.";

/// `trousseau init`.
pub const INIT: &str = r#"Examples:
  A project store in this directory, encrypted to your own key:
    trousseau init

  Your personal store, used when no project store is found:
    trousseau init --global

  A store shared with a teammate's age key and an SSH key:
    trousseau init --recipient age1... --recipient "ssh-ed25519 AAAA..."

  Recipients from a file, one per line ('#' comments allowed):
    trousseau init --recipients-file team.txt

  A store protected by a passphrase instead of keys:
    trousseau init --passphrase

Notes:
  init refuses to overwrite an existing store (exit 8).
  The store file is encrypted, so committing '.trousseau' to git is fine.
  Your identity file is the private half. Never commit or share it."#;

/// `trousseau info`.
pub const INFO: &str = r"Examples:
  The store a command run from this directory would use:
    trousseau info

  Your personal store:
    trousseau info --global

  A specific store file, as JSON:
    trousseau info --store ./other.trousseau --json";

/// `trousseau set`.
pub const SET: &str = r#"Where the value comes from (first match wins):
  1. --from-file PATH   the file's bytes, verbatim ('-' reads stdin verbatim)
  2. --from-env NAME    the value of that environment variable
  3. a hidden prompt    when stdin is a terminal
  4. stdin              when piped; one trailing newline is stripped

Examples:
  Type the value at a hidden prompt:
    trousseau set database/password

  Pipe the value in:
    printf %s "$TOKEN" | trousseau set api/token

  Store a file, such as a TLS key (binary data is handled for you):
    trousseau set tls/server.key --from-file server.key

  Copy the value of an environment variable:
    trousseau set api/token --from-env API_TOKEN

  Choose the variable name 'run' and 'env' will use, and describe the entry:
    trousseau set db/password --env PGPASSWORD --description "Postgres role"

Notes:
  Setting an existing key replaces its value and keeps its --env and
  --description unless you pass them again.
  To change only --env or --description, use 'trousseau edit'."#;

/// `trousseau get`.
pub const GET: &str = r#"Examples:
  Print a secret:
    trousseau get database/password

  Copy it to the clipboard (cleared again after 45 seconds):
    trousseau get api/token --clip

  Write a binary value to a file (created with mode 0600):
    trousseau get tls/server.key --out server.key

  Use a secret in another command without printing it:
    PGPASSWORD="$(trousseau get db/password)" psql -h db.internal app

  Value plus metadata as JSON:
    trousseau get database/password --json

Notes:
  A missing key exits with code 5. 'trousseau ls' lists the keys.
  To hand many secrets to a program at once, see 'trousseau run'."#;

/// `trousseau ls`.
pub const LS: &str = r"Examples:
  Every key, one per line:
    trousseau ls

  Only 'database' and the keys under 'database/':
    trousseau ls database

  A table with encoding, explicit env name, update time, and description:
    trousseau ls --long

  The same data as JSON:
    trousseau ls --json

Notes:
  PREFIX matches whole path segments: 'ls data' does not list
  'database/password'.
  The ENV column of --long shows only names set with 'set --env'. Other
  entries get a name derived from the key; see 'trousseau help run'.";

/// `trousseau rm`.
pub const RM: &str = r"Examples:
  Remove one key:
    trousseau rm api/token

  Remove several keys at once:
    trousseau rm api/token database/password

  Ignore keys that do not exist:
    trousseau rm --force maybe/missing api/token";

/// `trousseau mv`.
pub const MV: &str = r"Examples:
  Rename a key:
    trousseau mv api/token api/github-token

  Replace the destination if it already exists:
    trousseau mv --force staging/db-password database/password

Notes:
  Without --force, an existing destination is an error (exit 8).
  An entry with no explicit --env name gets a new derived variable name
  after a rename, because the name comes from the key.";

/// `trousseau recipients`.
pub const RECIPIENTS: &str = r#"Examples:
  See who can open the store:
    trousseau recipients ls

  Give a teammate access with their age key:
    trousseau recipients add age1...

  Give access with an SSH public key (quote it, it contains spaces):
    trousseau recipients add "ssh-ed25519 AAAA... sam@laptop"

  Take access away:
    trousseau recipients rm age1...

Finding a recipient to share:
  Your age recipient is the '# public key:' line of your identity file,
  by default ~/.config/trousseau/identity.txt. 'trousseau init' also
  prints it when it creates the identity.
  An SSH recipient is the content of the .pub file, for example
  ~/.ssh/id_ed25519.pub.

Notes:
  Removing a recipient only affects future versions of the file. Someone
  who kept an old copy (git history, a backup) can still read that copy,
  so change the secrets too.
  Passphrase stores have no recipients. Convert one with
  'trousseau rekey --to-recipients'."#;

/// `trousseau recipients add`.
pub const RECIPIENTS_ADD: &str = r#"Examples:
  Add an age recipient:
    trousseau recipients add age1...

  Add several at once, mixing age and SSH keys:
    trousseau recipients add age1... "ssh-ed25519 AAAA... sam@laptop""#;

/// `trousseau recipients rm`.
pub const RECIPIENTS_RM: &str = r"Examples:
  Remove a recipient:
    trousseau recipients rm age1...

  Remove your own recipient without the confirmation prompt:
    trousseau recipients rm --force age1...";

/// `trousseau rekey`.
pub const REKEY: &str = r#"Examples:
  Re-encrypt with a fresh file key, same recipients or passphrase:
    trousseau rekey

  Change the passphrase of a passphrase store, or convert a recipients
  store to a passphrase store (prompts twice):
    trousseau rekey --to-passphrase

  Convert to a recipients store with exactly these recipients:
    trousseau rekey --to-recipients age1... "ssh-ed25519 AAAA... sam@laptop"

Notes:
  --to-recipients does not add you implicitly. Include your own recipient,
  or you will not be able to open the store afterward.
  To add or remove one recipient, 'trousseau recipients' is simpler."#;

/// `trousseau export`.
pub const EXPORT: &str = r#"Formats:
  json     the full document, with metadata (default); round-trips through
           'import'
  dotenv   NAME="value" lines; binary entries are skipped with a warning
  toml     the same document 'trousseau edit' opens

Examples:
  A full backup, as JSON, to a new file (mode 0600):
    trousseau export --out backup.json

  A .env file:
    trousseau export --format dotenv --out .env

  Copy every entry into another store:
    trousseau export | trousseau import --store ./other.trousseau

Notes:
  --out refuses to overwrite an existing file (exit 8) unless --force."#;

/// `trousseau import`.
pub const IMPORT: &str = r"Strategies, for keys that already exist:
  fail        abort the whole import, change nothing (default, exit 8)
  keep        keep the existing entry, skip the imported one
  overwrite   replace the existing entry with the imported one

Examples:
  Restore a JSON backup into an empty store:
    trousseau import backup.json

  Bring in an existing .env file, replacing keys that already exist:
    trousseau import --format dotenv --strategy overwrite .env

  Read from stdin:
    trousseau export --store ./old.store | trousseau import --strategy keep

Notes:
  dotenv import lowercases each NAME to make the key (DATABASE_URL becomes
  'database_url') and records NAME as the entry's explicit env name, so
  'run' and 'env' give back the exact variable you imported.";

/// `trousseau run`.
pub const RUN: &str = r"Variable names:
  A name is derived from the key: '/', '-' and '.' become '_', letters are
  uppercased, and a leading digit gets '_' in front.
    database/password   ->  DATABASE_PASSWORD
    api/v2.token        ->  API_V2_TOKEN
  An entry saved with 'set --env NAME' uses NAME instead.
  --env-prefix goes in front of every name, derived or explicit.

Which entries are injected:
  Every text entry, unless --only is given. Binary entries are skipped
  with a warning. --only takes a whole key or a path prefix, and can be
  repeated: '--only database' selects 'database' and everything under
  'database/', but not 'database-replica/password'.
  If the store also holds secrets this command should not see, use --only.

Examples:
  Run a server with every secret in its environment:
    trousseau run -- ./server --port 8080

  Inject only the entries under 'database/':
    trousseau run --only database -- psql

  Inject exactly two entries:
    trousseau run --only database/password --only api/token -- ./deploy.sh

  Prefix every variable (APP_DATABASE_PASSWORD, ...):
    trousseau run --env-prefix APP_ -- ./server

  Give the child a minimal environment: the injected variables plus PATH,
  HOME, TMPDIR, TERM, LANG and LC_*:
    trousseau run --no-inherit -- ./server

Notes:
  Injected variables win over variables already in your environment.
  Two entries that resolve to the same name stop the command before it
  starts (exit 8). Fix it with an explicit 'set --env NAME' on one of them.
  To preview names and values without running anything, use 'trousseau env'
  with the same --only and --env-prefix flags.";

/// `trousseau env`.
pub const ENV: &str = r#"Variable names:
  A name is derived from the key: '/', '-' and '.' become '_', letters are
  uppercased, and a leading digit gets '_' in front.
    database/password   ->  DATABASE_PASSWORD
  An entry saved with 'set --env NAME' uses NAME instead.
  --env-prefix goes in front of every name, derived or explicit.

Which entries are printed:
  Every text entry, unless --only is given. Binary entries are skipped
  with a warning. --only takes a whole key or a path prefix, and can be
  repeated.

Examples:
  Load every secret into the current bash or zsh session:
    eval "$(trousseau env)"

  The same in fish:
    trousseau env | source

  Load only the entries under 'database/':
    eval "$(trousseau env --only database)"

  In a direnv .envrc, load exactly two entries:
    eval "$(trousseau env --only database/password --only api/token)"

  Write a .env file for a tool that needs one (plaintext, do not commit):
    trousseau env --format dotenv > .env

  Read one variable with jq:
    trousseau env --format json | jq -r .DATABASE_PASSWORD

Notes:
  Two entries that resolve to the same name are an error (exit 8).
  To run a single command with the secrets, 'trousseau run' is safer:
  nothing stays behind in your shell."#;

/// `trousseau edit`.
pub const EDIT: &str = r#"The document has one table per entry, keyed by the entry's key:

  ["database/password"]
  value = "s3cr3t"
  env = "PGPASSWORD"                  # optional
  description = "Postgres app role"   # optional

  Add a table to create an entry. Delete a table to remove the entry.
  Binary entries use 'value_base64' instead of 'value'.

Examples:
  Edit with your default editor:
    trousseau edit

  Use VS Code for this one edit (--wait is required):
    EDITOR="code --wait" trousseau edit

Notes:
  edit is the way to change an entry's env name or description without
  retyping its value.
  Some editors keep swap or backup copies of what you open. For vim, add
  to ~/.vimrc:
    autocmd BufNewFile,BufRead trousseau-*.toml
      \ setlocal nobackup nowritebackup noswapfile"#;

/// `trousseau migrate`.
pub const MIGRATE: &str = r"Examples:
  Migrate into your personal store:
    trousseau migrate ~/.trousseau --global

  Migrate into a new store file of your choice:
    trousseau migrate ~/.trousseau --store ./team.trousseau

  Migrate into a passphrase store:
    trousseau migrate ~/.trousseau --global --passphrase

Notes:
  Without --global or --store, the new store is '.trousseau' in the
  current directory. Run from your home directory, that is the legacy file
  itself, and migrate stops with exit 8. Use --global or --store there.
  The legacy passphrase comes from --passphrase-file or a prompt.
  OpenPGP legacy stores need the 'gpg' binary (see --gpg).
  Keys with characters the new format does not allow are renamed, and
  every rename is printed.";

/// `trousseau completions`.
pub const COMPLETIONS: &str = r#"Examples:
  fish:
    trousseau completions fish > ~/.config/fish/completions/trousseau.fish

  zsh (a directory in your fpath):
    trousseau completions zsh > "${fpath[1]}/_trousseau"

  bash, from ~/.bashrc:
    source <(trousseau completions bash)"#;

/// `trousseau man`.
pub const MAN: &str = r"Examples:
  Write the top-level man page and read it:
    trousseau man > trousseau.1
    man ./trousseau.1

  The man page of one subcommand:
    trousseau man set > trousseau-set.1";

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use clap::Parser as _;

    use crate::cli::Cli;

    /// Every long-form help constant, paired with its name for failure
    /// messages.
    const ALL: &[(&str, &str)] = &[
        ("TOP", super::TOP),
        ("TOP_LONG", super::TOP_LONG),
        ("INIT", super::INIT),
        ("INFO", super::INFO),
        ("SET", super::SET),
        ("GET", super::GET),
        ("LS", super::LS),
        ("RM", super::RM),
        ("MV", super::MV),
        ("RECIPIENTS", super::RECIPIENTS),
        ("RECIPIENTS_ADD", super::RECIPIENTS_ADD),
        ("RECIPIENTS_RM", super::RECIPIENTS_RM),
        ("REKEY", super::REKEY),
        ("EXPORT", super::EXPORT),
        ("IMPORT", super::IMPORT),
        ("RUN", super::RUN),
        ("ENV", super::ENV),
        ("EDIT", super::EDIT),
        ("MIGRATE", super::MIGRATE),
        ("COMPLETIONS", super::COMPLETIONS),
        ("MAN", super::MAN),
    ];

    /// Every `trousseau ...` example line in every help constant must
    /// actually parse against the real CLI grammar, so a renamed flag or
    /// removed command breaks this test instead of silently going stale
    /// in `--help` output.
    #[test]
    fn every_example_parses() {
        for (name, text) in ALL {
            for raw_line in text.lines() {
                let line = raw_line.trim();
                for piece in line.split(" | ") {
                    let Some(command) = piece.strip_prefix("trousseau ") else {
                        continue;
                    };
                    // Drop a `> file` redirection, then a two-space run
                    // that starts an inline description (the quick
                    // start table).
                    let command = command.split(" > ").next().unwrap_or(command);
                    let command = command.split("  ").next().unwrap_or(command).trim();
                    let words = shell_words::split(command).unwrap_or_else(|err| {
                        panic!("{name}: failed to tokenize {command:?}: {err}")
                    });
                    let mut argv = vec!["trousseau".to_owned()];
                    argv.extend(words);
                    Cli::try_parse_from(&argv)
                        .unwrap_or_else(|err| panic!("{name}: {raw_line:?} does not parse: {err}"));
                }
            }
        }
    }

    /// Style rules from `docs/HELP_TEXT_PLAN.md` section 4: no line
    /// wider than 80 columns, no trailing whitespace, no trailing
    /// newline, no em dash, no leftover spec reference.
    #[test]
    fn help_text_style() {
        for (name, text) in ALL {
            assert!(!text.ends_with('\n'), "{name} ends with a newline");
            assert!(!text.contains('\u{2014}'), "{name} contains an em dash");
            assert!(!text.contains("(3."), "{name} contains a spec reference");
            for (idx, line) in text.lines().enumerate() {
                assert!(
                    line.chars().count() <= 80,
                    "{name} line {}: {line:?} is wider than 80 columns",
                    idx + 1
                );
                assert_eq!(
                    line,
                    line.trim_end(),
                    "{name} line {}: {line:?} has trailing whitespace",
                    idx + 1
                );
            }
        }
    }
}
