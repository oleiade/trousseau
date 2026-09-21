# Command-line interface

This is the reference for the `trousseau` command: global behavior, exit
codes, every subcommand, and the JSON output shapes. It restates
`docs/IMPLEMENTATION_PLAN.md` section 3.5 and appendix 5.1; that document is
the source of truth and this one must not contradict it.

Status: implemented.

## Global behavior

- Binary name: `trousseau`. `--version` prints `trousseau <semver>`.
- Global flags, valid before or after the subcommand: `--store PATH`,
  `--global`, `--identity PATH` (repeatable), `--passphrase-file PATH`,
  `--json`, `--quiet`, `--no-input`.
- `--json`: read commands emit exactly one JSON document on stdout and
  nothing else on stdout. Write commands emit `{"ok": true, ...}` on
  success. Errors in `--json` mode go to stderr as
  `{"error": {"code": "<snake_case>", "message": "..."}}` and stdout stays
  empty.
- `--quiet`: suppress informational stderr lines. Errors still print.
- `--no-input`: never prompt. Anything that would have prompted fails with
  exit code 4 (secret input) or 2 (confirmation).
- stdin not a terminal implies `--no-input` for confirmations, but secret
  values are still read from stdin where the command says so.
- Colors: none. trousseau does not depend on a color library.
- Panics: `human-panic` is configured with a report path; the report never
  includes environment variables or arguments.
- Every command's `--help` ends with worked examples and, where relevant,
  notes on selection or naming rules; `-h` shows a shorter version of the
  same. This text is snapshot tested in
  `crates/trousseau-cli/tests/help.rs`.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Generic failure (I/O, invalid store, unexpected) |
| 2 | Usage error (clap), refused confirmation, `--no-input` on a confirmation |
| 3 | Store not found |
| 4 | Cannot unlock: no identity, wrong passphrase, no matching identity |
| 5 | Key not found |
| 6 | Lock timeout (another trousseau process holds the store) |
| 7 | Legacy v0.4 store detected; run `migrate` |
| 8 | Conflict: key exists (`mv` without `--force`), store exists (`init`), env name conflict |
| child | `run` exits with the child's code; on Unix, a child killed by signal N exits 128+N |

## Locking

Read commands take a shared lock, write commands an exclusive lock, for the
whole duration of the command. Lock wait is 5 seconds, then exit 6. Lock
files live in the cache directory (see `docs/format.md`).

## Timestamps

`updated_at` on the store and on the entry are set to now (UTC, seconds) on
every mutation. `created_at` never changes.

## `init`

```
trousseau init [--recipient R]... [--recipients-file PATH] [--passphrase] [--no-self]
```

- Refuses if the target store exists: exit 8.
- `--passphrase`: creates a passphrase store. Mutually exclusive with
  `--recipient`, `--recipients-file`, `--no-self`.
- Otherwise creates a recipients store. Recipients are the union of
  `--recipient` values, lines of `--recipients-file` (blank lines and `#`
  comments ignored), and, unless `--no-self`, the user's own recipient.
- The user's own recipient comes from the default identity file. If that
  file does not exist, `init` generates an X25519 identity with
  `age::x25519::Identity::generate()`, writes it to the default identity
  path with mode 0600 (creating the directory 0700), and prints on stderr:
  `created identity <path>` and `your recipient: age1...`.
- If interactive and `~/.ssh/id_ed25519.pub` exists and is not already in
  the list, ask: `Also encrypt to your SSH key ~/.ssh/id_ed25519.pub? [y/N]`.
  Default no.
- With `--no-self` and no recipients: exit 2 with
  `at least one recipient is required`.
- Writes an empty store (`entries: {}`) and prints `created <path>` on
  stderr. In `--json` mode:
  `{"ok": true, "path": "...", "kind": "recipients", "recipients": [...]}`.
- Never touches `.gitignore`.

## `info`

```
trousseau info
```

Prints, without decrypting when possible:

```
path:        /home/t/project/.trousseau
kind:        recipients
schema:      1
recipients:  2
entries:     14
updated:     2026-09-12T10:02:13Z
```

`kind` comes from the age header. The rest needs decryption; if unlocking
fails, `info` prints `path` and `kind` and the lines `schema: (locked)`
etc., and exits 0.

`--json`:
`{"path", "kind", "schema", "recipients", "entries", "updated_at", "locked": bool}`.

## `set`

```
trousseau set KEY [--from-file PATH | --from-env NAME] [--binary] [--env NAME] [--description TEXT]
```

Value source, exactly one:

1. `--from-file PATH`: file bytes verbatim. `-` means stdin, verbatim, no
   newline stripping.
2. `--from-env NAME`: the named environment variable's value. Missing
   variable: exit 1.
3. Neither, stdin is a terminal: hidden prompt `Value for KEY: `. No
   confirmation.
4. Neither, stdin is not a terminal: read all of stdin, then strip exactly
   one trailing `\n` or `\r\n`.

A second positional argument (`set KEY VALUE`) is refused with exit code
2 and an explanation of the four sources above; the value itself is
never echoed, on either stdout or stderr.

Encoding: `base64` if `--binary` or if the bytes are not valid UTF-8 or
contain NUL; otherwise `utf8`.

Existing key: keep `created_at`, keep `env` and `description` unless the
flag is given; set `updated_at`. Empty values are allowed.

Output: `set KEY` on stderr unless `--quiet`.

`--json`: `{"ok": true, "key": "...", "encoding": "utf8", "created": bool}`.

## `get`

```
trousseau get KEY [--out PATH] [--force] [--clip]
```

- Default: raw value bytes to stdout. If stdout is a terminal and encoding
  is `utf8`, `get` appends `\n`. If stdout is a terminal and encoding is
  `base64`, it refuses with `binary value; use --out or pipe the output`,
  exit 1. If stdout is not a terminal, `get` writes raw bytes with no
  newline in either case.
- `--out PATH`: write bytes to the file with mode 0600. If the file exists,
  exit 8 unless `--force` is given.
- `--clip`: copy to the clipboard, print
  `copied KEY to clipboard, clearing in 45s` on stderr, and schedule
  clearing (see "Clipboard clearing" below). Requires the `clipboard`
  feature; otherwise exit 1 with `built without clipboard support`.
- `--json`:
  `{"key", "value", "encoding", "env", "description", "created_at", "updated_at"}`
  where `value` is the stored representation (utf8 string or base64
  string).
- Missing key: exit 5, message `key not found: KEY`.

## `ls`

```
trousseau ls [PREFIX] [--long]
```

- Lists keys sorted bytewise, one per line.
- `PREFIX` filters to keys equal to `PREFIX` or starting with `PREFIX/`. It
  is a path prefix, not a string prefix: `ls data` does not list
  `database/password`.
- `--long`: a table with columns `KEY`, `ENC`, `ENV`, `UPDATED`,
  `DESCRIPTION`. Never values.
- `--json`: array of
  `{"key", "encoding", "env", "description", "created_at", "updated_at"}`.
- An empty result is exit 0 with empty output.

## `rm`

```
trousseau rm KEY... [--force]
```

Removes each key. Any missing key: exit 5 and nothing is written, unless
`--force`, which ignores missing keys. Output per removed key on stderr:
`removed KEY`.

`--json`: `{"ok": true, "removed": [...]}`.

## `mv`

```
trousseau mv OLD NEW [--force]
```

Renames, keeping all entry metadata and `created_at`. `OLD` missing: exit
5. `NEW` exists: exit 8 unless `--force`.

`--json`: `{"ok": true, "from": "...", "to": "..."}`.

## `recipients`

```
trousseau recipients ls
trousseau recipients add R...
trousseau recipients rm R...
```

- Only valid on recipients stores; on a passphrase store: exit 1 with
  `this is a passphrase store; use 'rekey --to-recipients'`.
- `ls`: one recipient per line as stored. `--json`: array of strings.
- `add`: validates each recipient (see `docs/format.md`), ignores
  duplicates with a stderr note, saves. Saving re-encrypts to the new set.
- `rm`: removes by exact match or by key material match (SSH comment
  differences do not matter). Refuses to remove the last recipient: exit
  2. If the removed recipient corresponds to one of the caller's own
  identities, `recipients rm` prints
  `warning: you removed your own recipient; you will not be able to open this store after this command`
  and requires confirmation (interactive) or `--force`.
- `--json` for `add`/`rm`: `{"ok": true, "recipients": [...]}`.

## `rekey`

```
trousseau rekey [--to-passphrase | --to-recipients R... ]
```

- No flags: re-encrypt to the current recipient set (or passphrase) with a
  fresh file key. Useful after a suspected leak of the file.
- `--to-passphrase`: convert to a passphrase store; prompt twice;
  `recipients` becomes empty and `kind` becomes `passphrase`.
- `--to-recipients R...`: convert to a recipients store with exactly the
  listed recipients (the user's own recipient is not added implicitly).
- `--json`: `{"ok": true, "kind": "...", "recipients": [...]}`.

## `export` and `import`

```
trousseau export [--format json|dotenv|toml] [--out PATH]
trousseau import [--format json|dotenv|toml] [--strategy keep|overwrite|fail] [PATH]
```

- `json` (default): the full payload document (see `docs/format.md`).
  `import --format json` accepts the same document and merges its
  `entries`; it ignores `recipients`, `kind` and the store-level
  timestamps of the imported document. Imported entries keep their
  `created_at` and get `updated_at` = now.
- `dotenv`: one `NAME="value"` line per utf8 entry using the entry's
  resolved env name (no prefix); `"`, `\` and newline are escaped as `\"`,
  `\\`, `\n`. base64 entries are skipped with a stderr warning.
  `import --format dotenv` parses `NAME=value`, `NAME="value"` and
  `NAME='value'` lines, ignores blank and `#` lines, sets key = lowercase
  of NAME with `_` kept, and sets `env` = NAME.
- `toml`: the `edit` document format (see below). Import parses the same.
- `--strategy` for `import`: `keep` (existing keys win), `overwrite`
  (imported wins), `fail` (any collision aborts with exit 8). Default
  `fail`.
- `export` writes to stdout; `--out PATH` writes a 0600 file and exits 8
  if the file exists, unless `--force`.
- `export` always requires unlocking. It prints a one-line stderr warning
  when exporting plaintext to a file:
  `warning: <path> contains plaintext secrets`.
- `--json` for `export`: the payload document (format `json`), or rejected
  with exit 2 for other formats.
- `--json` for `import`: `{"ok":true,"added":N,"updated":N,"skipped":N}`.

## `edit`

```
trousseau edit
```

Document format (TOML), generated with `toml` and a header comment:

```toml
# trousseau edit: /home/t/project/.trousseau
# Save and quit to apply. Delete a table to remove its entry.
# Leave the file empty to abort.

["database/password"]
value = "s3cr3t"
env = "DATABASE_PASSWORD"
description = "Postgres app role"

["tls/server.key"]
value_base64 = "LS0tLS1CRUdJTi4uLg=="
```

Rules:

- Exactly one of `value` (utf8) or `value_base64` per table. Tables are
  keyed by the entry key.
- Scratch file: created with `tempfile::Builder` with prefix `trousseau-`
  and suffix `.toml`, mode 0600, in the first existing directory of:
  `$XDG_RUNTIME_DIR`, `/dev/shm`, the system temp dir. On Windows: the
  system temp dir.
- Editor: `$VISUAL`, else `$EDITOR`, else `vi` on Unix and `notepad` on
  Windows. The command line is split with `shell-words`. It runs with
  stdin, stdout, stderr inherited. A non-zero editor exit aborts, exit 1,
  scratch deleted.
- After the editor exits: if the file is empty or unchanged, `edit` prints
  `no changes` and exits 0. Otherwise it parses the file; on a parse error
  it prints the TOML error with line number and, if interactive, asks
  `Reopen the editor? [Y/n]`; otherwise it exits 1. The scratch file is
  deleted in every path, including panics (a guard with `Drop` handles
  this).
- Apply: removed tables remove entries; changed values or metadata bump
  `updated_at`; unchanged entries keep their timestamps; new tables create
  entries with both timestamps set to now.
- `--json`: `{"ok": true, "changed": bool}` on stdout, whether or not the
  file actually differed from the store.

### Editor advice

`edit` writes a plaintext scratch copy of your secrets to disk for the
duration of the editor session, and deletes it afterward. Some editors
leave their own backup or swap copies behind; disable that for trousseau's
scratch files.

**vim**: add this to your `~/.vimrc`, scoped to the scratch file pattern
so it does not change behavior for other files:

```vim
autocmd BufNewFile,BufRead trousseau-*.toml setlocal nobackup nowritebackup noswapfile
```

**VS Code**: run `code --wait` as your `$EDITOR` (or `$VISUAL`) so
trousseau waits for you to close the tab before it reads the file back:

```sh
export EDITOR="code --wait"
```

VS Code's local history and hot-exit features can still retain a copy of
the scratch file's contents outside the scratch path itself; be aware of
this if VS Code is your editor for `trousseau edit`.

## `run`

```
trousseau run [--env-prefix P] [--only KEYPREFIX]... [--no-inherit] -- CMD [ARGS]...
```

- Unlocks the store, builds the environment (see `docs/format.md`), and
  executes `CMD`.
- `--env-prefix P` defaults to `[run].env_prefix` from the configuration
  file (`docs/format.md`'s "Configuration file") when absent; the flag
  overrides the config value when given.
- Selection: all entries, or only those under any `--only` path prefix
  (same semantics as `ls PREFIX`).
- base64 entries are skipped with one stderr warning per entry:
  `skipping binary entry KEY`.
- `--no-inherit`: the child gets only the injected variables plus `PATH`,
  `HOME`, `TMPDIR`, `TERM`, `LANG`, `LC_*` if set. Default: inherit the
  parent's environment with injected variables overriding.
- Env name conflicts: exit 8 before executing anything.
- Unix: `run` replaces the current process with
  `std::os::unix::process::CommandExt::exec`. If `exec` returns, it prints
  the error and exits 1. Windows: `run` spawns, waits, and exits with the
  child's code.
- Nothing is written to disk. No lock is held while the child runs: the
  shared lock is released after decryption, before `exec`.
- `--json` is rejected for `run` (exit 2).

## `env`

```
trousseau env [--format shell|dotenv|json] [--env-prefix P] [--only KEYPREFIX]...
```

- `shell` (default): `export NAME='value'` lines with single-quote
  escaping (`'` becomes `'\''`). Safe for `eval "$(trousseau env)"` and for
  direnv's `.envrc`.
- `dotenv`: as in `export`/`import`.
- `json`: object `{"NAME": "value", ...}`. `--json` is an alias for
  `--format json`.
- Same selection, skipping, conflict, and `--env-prefix` default rules as
  `run`.

## `migrate`

```
trousseau migrate SOURCE [--gpg PATH] [--gnupg-home PATH] [--new-passphrase-file PATH]
```

See `docs/migration.md` for the end-user walkthrough. In short:

- `SOURCE` is a v0.4 store file (usually `~/.trousseau`). The target is the
  store resolved by the normal rules; it must not already exist (exit 8).
  Recipients or passphrase for the target follow the `init` rules, so
  `migrate` accepts the same `--recipient`, `--recipients-file`,
  `--passphrase`, `--no-self` flags.
- Not a legacy store: exit 1 with `not a v0.4 store`.
- AES stores: the legacy passphrase comes from `--passphrase-file`,
  `TROUSSEAU_PASSPHRASE`, or a hidden prompt, in that order. When the
  target is also a passphrase store, the new passphrase comes from
  `--new-passphrase-file` or a separate double prompt
  `New passphrase for the migrated store: `. The two passphrases are never
  assumed equal.
- OpenPGP stores: `migrate` runs the `gpg` binary. Not found: exit 1 naming
  the binary.
- Key sanitization: legacy keys may contain any characters. Each key is
  converted by replacing every character outside `[A-Za-z0-9._/-]` with
  `_`, collapsing repeated `/`, trimming leading and trailing `/`, and, if
  the result is empty or invalid, using `migrated/<index>`. Collisions get
  `_2`, `_3` suffixes. Every renamed key is printed:
  `renamed "easy as" -> easy_as`.
- Legacy `recipients` (PGP key ids) are printed for information and not
  carried.
- The source file is never modified or deleted.
- Output: `migrated N entries to <path>`.
- `--json`:
  `{"ok": true, "path", "entries": N, "renamed": [{"from","to"}], "legacy_recipients": [...]}`.

## Clipboard clearing

`get --clip` copies the value, then spawns a detached copy of itself:
`trousseau __clip-clear <sha256-hex-of-value> <seconds>` (a hidden
subcommand, not shown in help). That process sleeps, reads the clipboard,
and clears it only if the clipboard's sha256 still matches. On Unix, it
detaches with `setsid`-like semantics via
`Command::new(current_exe).process_group(0)` and redirected stdio to null.
The value itself is never passed on the command line; only its hash is.

## `completions` and `man`

`trousseau completions <bash|zsh|fish|powershell|elvish>` prints a
completion script from `clap_complete`. `trousseau man` prints the roff
page for the top-level command; `trousseau man <subcommand>` prints it for
a subcommand. Both are used by the release pipeline to generate packaged
files.

## Verify a download

Every release asset (the archives, the `.deb` and `.apk` packages, and
`checksums.txt`) is attested by `actions/attest-build-provenance` and
carries a checksum. Verify both before trusting a downloaded file.

Attestation (requires the [GitHub CLI](https://cli.github.com/), `gh auth
login` once):

```
gh attestation verify trousseau_1.0.0_linux_amd64.tar.gz --owner oleiade
```

This confirms the file was built by the `release.yml` workflow from a
specific commit and workflow run, not tampered with afterward.

Checksum, against the `checksums.txt` asset published alongside the
release:

```
sha256sum -c checksums.txt --ignore-missing
```

`--ignore-missing` lets you run this from a directory that only holds
the one or two files you downloaded, rather than every platform's
archive.

### Reading the SBOM

Each release also carries `trousseau.cdx.json`, a
[CycloneDX](https://cyclonedx.org/) software bill of materials for the
`trousseau` binary, generated by `cargo cyclonedx`. It is plain JSON: a
`metadata.component` describing the binary itself, and a `components`
array with every dependency, its version, and its license. Skim it with
`jq`:

```
jq -r '.components[] | "\(.name) \(.version)"' trousseau.cdx.json
```

or load it into any CycloneDX-aware SBOM viewer or vulnerability
scanner.

## JSON output shapes

| Command | stdout in `--json` mode |
|---|---|
| `init` | `{"ok":true,"path":"...","kind":"recipients","recipients":["..."]}` |
| `info` | `{"path":"...","kind":"...","schema":1,"recipients":2,"entries":14,"updated_at":"...","locked":false}` |
| `set` | `{"ok":true,"key":"...","encoding":"utf8","created":true}` |
| `get` | `{"key":"...","value":"...","encoding":"...","env":null,"description":null,"created_at":"...","updated_at":"..."}` |
| `ls` | `[{"key":"...","encoding":"...","env":null,"description":null,"created_at":"...","updated_at":"..."}]` |
| `rm` | `{"ok":true,"removed":["..."]}` |
| `mv` | `{"ok":true,"from":"...","to":"..."}` |
| `recipients ls` | `["..."]` |
| `recipients add/rm` | `{"ok":true,"recipients":["..."]}` |
| `rekey` | `{"ok":true,"kind":"...","recipients":["..."]}` |
| `export` | the payload document (format json), or rejected with exit 2 for other formats |
| `import` | `{"ok":true,"added":N,"updated":N,"skipped":N}` |
| `edit` | `{"ok":true,"changed":bool}` |
| `env` | `{"NAME":"value"}` |
| `migrate` | `{"ok":true,"path":"...","entries":N,"renamed":[{"from":"...","to":"..."}],"legacy_recipients":["..."]}` |
| errors | stderr: `{"error":{"code":"key_not_found","message":"key not found: x"}}` |

Error `code` values are the `Error` variant names in snake_case, plus
`usage`, `refused`, `child_failed`.

## Test-only environment variables

Honored only in debug builds (`#[cfg(debug_assertions)]`), rejected
otherwise:

| Variable | Effect |
|---|---|
| `TROUSSEAU_TEST_NOW` | RFC 3339 timestamp used as "now". |
| `TROUSSEAU_TEST_LOCK_TIMEOUT_MS` | Overrides the 5 s lock timeout. |
