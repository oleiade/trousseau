# Trousseau Rust rewrite: implementation plan

Status: normative. Version 1, 2026-09-19.
Owner and reviewer: Théo Crevon (oleiade).
Implementer: an AI coding agent working one step at a time, or a human following the same steps.

This document turns the approved design (see `docs/design/rebuilding-trousseau.html`, the plan page) into an ordered sequence of pull requests. Every step below produces exactly one PR, stacked on the previous one, so the reviewer can read the rewrite in order.

---

## 0. How to read and use this document

### 0.1 Language

- MUST and MUST NOT are hard requirements. A PR that violates one is not mergeable.
- SHOULD is the default; deviate only with a one-line reason in the PR description.
- "The agent" is whoever implements a step. "The reviewer" is Théo.
- Step identifiers look like `2.3`. Branch names, PR titles and commit scopes reference them.

### 0.2 Non-negotiable rules for every step

1. Work only on the `rust-rewrite` integration branch and on `rr/*` step branches. Never commit to `master`.
2. One step, one PR. Do not bundle two steps. Do not start step N+1 in step N's branch.
3. Every PR MUST be green on CI before it is opened for review: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo deny check`, `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"`.
4. No `unsafe`. No `unwrap()` or `expect()` outside tests. No `panic!` in library code. The lint configuration in section 2.4 enforces this; do not weaken it.
5. Secret values MUST NOT appear in: command-line arguments, log lines, error messages, `Debug` output, panics, temporary files other than the ones section 3.5.12 (`edit`) specifies.
6. Do not add a dependency that is not in the table in section 2.3 without listing it in the PR description with the reason and a link to the crate.
7. Do not implement anything this document does not specify. If a feature seems missing, write it down under "Follow-ups" in the PR description and move on.
8. Read the documentation on docs.rs for the pinned version of every crate before using it. This document names API areas, not exact signatures; verify them.
9. Write tests in the same PR as the code they test. A step without its tests is incomplete.
10. Commit messages use Conventional Commits (`feat(cli): add get command`) and end with the attribution trailer the session tooling requires. PR bodies end with the attribution line the session tooling requires.

### 0.3 Definition of done for a step

A step is done when all of the following are true:

- Every deliverable file listed for the step exists with the specified content or behavior.
- Every test listed for the step exists and passes on Linux, macOS and Windows in CI (Windows exceptions are called out explicitly per step).
- The acceptance criteria for the step pass when run by hand from a clean checkout.
- The PR description follows the template in section 1.6.
- CI is green.

---

## 1. Repository, branches, and stacked pull requests

### 1.1 Branch layout

| Branch | Purpose | Who creates it |
|---|---|---|
| `master` | Untouched until the final merge. Holds the Go history. | Exists |
| `legacy-go` | Frozen copy of `master` at the start of the rewrite. Never receives commits. | Reviewer (step 0.0) |
| Tag `go-final` | Same commit as `legacy-go`. Used by scripts that need the Go binary. | Reviewer (step 0.0) |
| `rust-rewrite` | Integration branch. Every step PR eventually lands here. | Reviewer (step 0.0) |
| `rr/<step>-<slug>` | One branch per step, e.g. `rr/2.2-envelope`. | Agent |

### 1.2 How the stack works

- Step 0.1's branch is created from `rust-rewrite`. Every later step's branch is created from the previous step's branch.
- Each PR's base is the previous step's branch. The first PR's base is `rust-rewrite`.
- The reviewer merges PRs bottom-up, using "Create a merge commit". Squash merging MUST NOT be used on this stack: squashing rewrites history and forces every open PR above to be rebased.
- After merging a PR, the reviewer deletes its head branch. GitHub then retargets the next open PR in the stack onto the merged PR's base automatically. The agent verifies the base after each merge and fixes it with `gh pr edit <n> --base rust-rewrite` if needed.
- If the reviewer requests changes on step N while steps N+1.. are open: fix on `rr/N-*`, push, then rebase every open branch above it onto the updated parent, in order.

### 1.3 Commands with git and gh

Create the step branch and PR (replace names):

```bash
git switch rr/2.1-schema            # the parent step's branch
git switch -c rr/2.2-envelope
# ... work, commit ...
git push -u origin rr/2.2-envelope
gh pr create --base rr/2.1-schema --head rr/2.2-envelope \
  --title "feat(lib): age envelope (step 2.2)" --body-file .github/PR_BODY.md
```

Rebase the stack after a parent changed:

```bash
git switch rr/2.3-store-io
git rebase --onto rr/2.2-envelope <old-parent-sha> rr/2.3-store-io
git push --force-with-lease
```

### 1.4 Commands with jujutsu

The repository is used colocated (`jj git init --colocate` once in an existing clone). Bookmarks map to branches.

```bash
jj new rr/2.1-schema                       # start a change on top of the parent step
# ... work ...
jj describe -m "feat(lib): age envelope (step 2.2)"
jj bookmark create rr/2.2-envelope -r @
jj git push --bookmark rr/2.2-envelope
gh pr create --base rr/2.1-schema --head rr/2.2-envelope --title "..." --body-file ...
```

Rebasing the whole stack after a parent changed is one command: `jj rebase -s rr/2.3-store-io -d rr/2.2-envelope`, then `jj git push --all`.

### 1.5 PR sizing

Target under 600 changed lines of Rust per PR, tests included. Steps in this document are sized for that. If a step grows past 900 lines, stop, split it into `N.Ma` and `N.Mb` with the same rules, and note the split in both PR descriptions.

### 1.6 PR description template

Commit this file as `.github/PULL_REQUEST_TEMPLATE.md` in step 1.3. Until then, paste it.

```markdown
## Step

<step id and name from docs/IMPLEMENTATION_PLAN.md>

## What

<two to five sentences: what this PR adds or removes>

## Why

<one paragraph, or "See plan section X">

## How to review

<ordered list of files to read first, and what to look for>

## Test evidence

<paste of the test summary line, plus manual acceptance commands and their output>

## Deviations from the plan

<"None" or a list with reasons>

## Follow-ups

<"None" or a list of things noticed but out of scope>
```

### 1.7 The final merge

After step 5.4, one PR merges `rust-rewrite` into `master` with a merge commit. `master` keeps the full Go history under it. The Go code stays reachable through the `legacy-go` branch and the `go-final` tag.

---

## 2. Target architecture

### 2.1 Repository layout at the end of Phase 3

```
.
├── Cargo.toml                  # workspace
├── Cargo.lock                  # committed
├── rust-toolchain.toml         # channel = "stable"
├── rustfmt.toml
├── deny.toml
├── renovate.json
├── justfile
├── LICENSE
├── README.md
├── CHANGELOG.md
├── SECURITY.md
├── AUTHORS.md
├── docs/
│   ├── IMPLEMENTATION_PLAN.md  # this file
│   ├── format.md               # store format specification
│   ├── cli.md                  # command reference
│   ├── threat-model.md
│   ├── migration.md
│   └── design/rebuilding-trousseau.html
├── scripts/
│   └── legacy/generate-fixtures.sh
├── crates/
│   ├── trousseau/              # library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   ├── schema.rs       # Store, Entry, Key, Encoding, env mapping
│   │   │   ├── envelope.rs     # age seal/open
│   │   │   ├── identity.rs     # recipients and identities
│   │   │   ├── store.rs        # locate, lock, read, atomic write
│   │   │   └── legacy.rs       # v0.4 reader
│   │   └── tests/
│   │       └── fixtures/legacy/
│   └── trousseau-cli/          # binary `trousseau`
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── cli.rs          # clap definitions
│       │   ├── config.rs       # config.toml
│       │   ├── context.rs      # resolved paths, identities, output mode
│       │   ├── output.rs       # human and JSON output helpers
│       │   ├── prompt.rs       # hidden input, confirmations, age Callbacks
│       │   ├── exit.rs         # exit codes
│       │   └── commands/
│       │       ├── mod.rs
│       │       ├── init.rs
│       │       ├── info.rs
│       │       ├── set.rs
│       │       ├── get.rs
│       │       ├── ls.rs
│       │       ├── rm.rs
│       │       ├── mv.rs
│       │       ├── recipients.rs
│       │       ├── rekey.rs
│       │       ├── export.rs
│       │       ├── import.rs
│       │       ├── run.rs
│       │       ├── env.rs
│       │       ├── edit.rs
│       │       ├── migrate.rs
│       │       ├── clip.rs
│       │       └── completions.rs
│       └── tests/              # assert_cmd integration tests
└── .github/
    ├── workflows/ci.yml
    ├── workflows/security.yml
    ├── workflows/release.yml
    ├── PULL_REQUEST_TEMPLATE.md
    └── CODEOWNERS
```

### 2.2 Crate responsibilities

- `trousseau` (library): everything that touches the format, the cryptography, the filesystem representation of a store, and the legacy format. It has no terminal I/O, no prompts, no process spawning except `gpg` in `legacy.rs`, and no knowledge of environment variables. It never prints.
- `trousseau-cli` (binary): argument parsing, configuration, prompting, output formatting, process execution for `run`, editor and clipboard integration, exit codes.

The boundary matters for review: anything cryptographic or format-related in the CLI crate is a defect.

### 2.3 Dependencies

Versions are minimums known to exist. Use the latest compatible release at implementation time and let Renovate track them after step 1.3.

| Crate | Where | Version | Features | Purpose |
|---|---|---|---|---|
| `age` | lib | 0.11 | `armor`, `ssh`, `plugin` | Envelope format. A caret requirement on the minor (`0.11`) is fine; do not use an exact pin. |
| `secrecy` | lib, cli | 0.10 | `serde` | `SecretString`, `SecretBox` wrappers. |
| `zeroize` | lib | 1.8 | `derive` | Zero secret buffers on drop. |
| `serde` | lib, cli | 1 | `derive` | Payload model. |
| `serde_json` | lib, cli | 1 | `preserve_order` NOT enabled | Payload storage. Output uses `BTreeMap` for order. |
| `toml` | cli | 0.8 | default | Config file, `edit` surface, `export --format toml`. |
| `time` | lib, cli | 0.3 | `formatting`, `parsing`, `serde-well-known`, `macros` | RFC 3339 timestamps. |
| `base64` | lib, cli | 0.22 | default | Binary values. |
| `thiserror` | lib | 2 | | Typed errors. |
| `anyhow` | cli | 1 | | Error context in the binary. |
| `sha2` | lib | 0.10 | | Lock file naming. Already in age's tree. |
| `tempfile` | lib, cli | 3 | | Atomic writes, `edit` scratch file. |
| `fd-lock` | lib | 4 | | Advisory locks. |
| `etcetera` | cli | 0.8 | | XDG and platform directories. |
| `clap` | cli | 4.5 | `derive`, `env`, `wrap_help` | CLI. |
| `clap_complete` | cli | 4.5 | | Shell completions. |
| `clap_mangen` | cli | 0.2 | | Man pages. |
| `rpassword` | cli | 7 | | Hidden input. |
| `dialoguer` | cli | 0.11 | default | Confirmations. |
| `arboard` | cli, optional | 3 | `default-features = false` | Clipboard, behind feature `clipboard` (default on). |
| `shell-words` | cli | 1 | | Split `$EDITOR`. |
| `human-panic` | cli | 2 | `default-features = false` | Panic reports without secrets (see 3.5.1 for the rule). |
| `scrypt` | lib | 0.11 | | Legacy KDF only. |
| `aes` | lib | 0.8 | | Legacy cipher only. |
| `cfb-mode` | lib | 0.8 | | Legacy cipher mode only. |
| `assert_cmd` | cli dev | 2 | | Integration tests. |
| `predicates` | cli dev | 3 | | Assertions on output. |
| `proptest` | lib dev | 1 | | Round-trip properties. |
| `insta` | cli dev | 1 | `json` | Snapshot tests on `--json` output. |
| `tempfile` | dev | 3 | | Test directories. |

Not allowed without a written reason: `tokio` or any async runtime, `reqwest` or any HTTP client, `openssl`, `ring`, `libc` direct usage, `nix`.

### 2.4 Lints and toolchain

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

Both crates' `Cargo.toml` carry `edition = "2024"`, `rust-version = "1.89"` (raised from 1.85 during step 2.1 because `aes` 0.9 and the patched `time` 0.3.46+ require it), and this block, copied verbatim from motus and extended:

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
enum_glob_use = "deny"
pedantic = "deny"
nursery = "deny"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
print_stdout = "deny"   # library crate only; the CLI allows it in output.rs via allow attributes
print_stderr = "deny"   # library crate only
```

In the CLI crate, `print_stdout` and `print_stderr` are set to `"warn"` at the crate level and `#[allow]`ed only inside `output.rs`. All user-facing printing goes through `output.rs`.

`rustfmt.toml`: `edition = "2024"`, `use_field_init_shorthand = true`, `imports_granularity` NOT set (unstable).

---

## 3. Specifications

Everything in this section is normative. The `docs/*.md` files committed in step 1.2 restate it for end users; this section is the source.

### 3.1 Store file format (schema 1)

#### 3.1.1 Envelope

A store is one file. Its bytes are an age file in ASCII armor: it begins with `-----BEGIN AGE ENCRYPTED FILE-----` and ends with `-----END AGE ENCRYPTED FILE-----` followed by a newline. Binary (unarmored) age files are NOT accepted as stores.

Two kinds of store exist, distinguished by the age header:

- Recipients store: one or more age recipient stanzas (X25519, ssh-ed25519, ssh-rsa, or plugin). The scrypt stanza MUST NOT be present.
- Passphrase store: exactly one scrypt stanza.

age itself forbids mixing an scrypt stanza with other stanzas. The library exposes `envelope::peek_kind` so the CLI can decide whether to prompt for a passphrase or load identities before decrypting.

Every save produces a fresh age file key. There is no in-place update.

#### 3.1.2 Payload

The decrypted payload is UTF-8 JSON, one object, serialized with `serde_json::to_vec_pretty`, keys in sorted order (use `BTreeMap` everywhere), with a trailing newline. Pretty printing is deliberate: an emergency `age -d` must produce something a human can read.

```json
{
  "schema": 1,
  "kind": "recipients",
  "created_at": "2026-09-12T09:41:00Z",
  "updated_at": "2026-09-12T10:02:13Z",
  "recipients": [
    "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p",
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHhX3Q6M1u4T4z0tR8O7x0Zk5rQ0p5s5H3o1Gv3nS6uM deploy@web-01"
  ],
  "entries": {
    "database/password": {
      "value": "s3cr3t",
      "encoding": "utf8",
      "env": "DATABASE_PASSWORD",
      "description": "Postgres app role",
      "created_at": "2026-09-12T09:41:00Z",
      "updated_at": "2026-09-12T09:41:00Z"
    },
    "tls/server.key": {
      "value": "LS0tLS1CRUdJTi4uLg==",
      "encoding": "base64",
      "created_at": "2026-09-12T09:50:00Z",
      "updated_at": "2026-09-12T09:50:00Z"
    }
  }
}
```

Field rules:

| Field | Type | Rule |
|---|---|---|
| `schema` | integer | MUST be `1`. A reader that sees a larger value MUST fail with `Error::SchemaTooNew { found }`. A smaller value cannot occur in schema 1. |
| `kind` | `"recipients"` or `"passphrase"` | `recipients` ⇒ `recipients` array non-empty. `passphrase` ⇒ `recipients` array empty. Any other combination is `Error::InvalidStore`. |
| `created_at`, `updated_at` | string | RFC 3339, UTC, second precision, `Z` suffix. |
| `recipients` | array of string | Each string is a valid recipient per 3.3.1, deduplicated, sorted. |
| `entries` | object | Keys satisfy 3.1.3. Sorted. |
| `entries.*.value` | string | For `utf8`: the value verbatim. For `base64`: standard base64 with padding (RFC 4648 section 4). |
| `entries.*.encoding` | `"utf8"` or `"base64"` | `utf8` requires the value to be valid UTF-8 without NUL bytes. Anything else is `base64`. |
| `entries.*.env` | string, optional | Overrides the derived environment variable name. Must match `^[A-Za-z_][A-Za-z0-9_]*$`. |
| `entries.*.description` | string, optional | Free text, max 1024 bytes. |
| `entries.*.created_at`, `updated_at` | string | As above. |

Unknown fields at any level are rejected (`#[serde(deny_unknown_fields)]`). This is how schema mistakes surface early. Schema 2, if it ever exists, changes the `schema` number.

Size limits: the payload MUST be at most 16 MiB; a single value at most 4 MiB. Exceeding them is `Error::TooLarge`.

#### 3.1.3 Keys

A key is a path of one or more segments separated by `/`.

- Segment grammar: `[A-Za-z0-9][A-Za-z0-9._-]*`.
- No empty segments, no leading or trailing `/`, no `.` or `..` segments.
- Total length at most 256 bytes. Case-sensitive. `Database/Password` and `database/password` are different keys.
- A key is never a prefix-parent of another key in a way that matters: `database` and `database/password` may both exist.

The `Key` newtype validates on construction and is the only way to build a key.

#### 3.1.4 Environment variable mapping

`env_name(key, prefix)`:

1. Take the key string.
2. Replace every `/`, `-` and `.` with `_`.
3. Uppercase ASCII letters.
4. If the first character is a digit, prepend `_`.
5. Prepend `prefix` verbatim if non-empty (the prefix is validated with the same regex as `env`).

An entry with an explicit `env` uses it unchanged, and the prefix is still prepended. Two entries that resolve to the same name are a conflict; `run` and `env` fail with `Error::EnvConflict { name, keys }` listing both keys.

### 3.2 Store discovery and paths

Directory roots come from `etcetera` with the XDG strategy on Linux and macOS (`~/.config`, `~/.local/share`, `~/.cache`), and the Windows strategy on Windows (`%APPDATA%`, `%LOCALAPPDATA%`). The macOS choice of XDG over `~/Library` is deliberate: dotfile users and CI runners expect it, and it matches what the age and sops ecosystems do.

| Item | Path |
|---|---|
| Config file | `<config_dir>/trousseau/config.toml` |
| Default identity | `<config_dir>/trousseau/identity.txt` |
| Personal store | `<data_dir>/trousseau/default.trousseau` |
| Lock files | `<cache_dir>/trousseau/locks/<sha256 hex of canonical store path>.lock` |
| Project store | `.trousseau` in the current directory or any ancestor |

Store selection order, first match wins:

1. `--store PATH`
2. `TROUSSEAU_STORE` (path)
3. If `--global` is present: the personal store.
4. The nearest `.trousseau` file walking up from the current directory to the filesystem root. Directories are checked, not files: `.trousseau` MUST be a regular file to match.
5. The personal store.

`init` uses the same order to decide where to create, except that rule 4 becomes "the current directory" (init never walks up). There is no verbose flag: `info` reports the resolved path, and every error message that concerns a store names its path.

File modes on Unix: store `0600`, identity `0600`, config `0600`, directories `0700`. On Windows, rely on the user profile ACLs; do not attempt ACL manipulation.

### 3.3 Recipients and identities

#### 3.3.1 Recipient strings

Accepted forms, detected by prefix:

| Form | Example | Parsed with |
|---|---|---|
| age X25519 | `age1...` (Bech32, 62 chars) | `age::x25519::Recipient::from_str` |
| SSH public key | `ssh-ed25519 AAAA... comment` or `ssh-rsa AAAA... comment` | `age::ssh::Recipient::from_str` |
| Plugin | `age1<plugin>1...` for example `age1yubikey1...` | `age::plugin::RecipientPluginV1` |

Anything else is `Error::InvalidRecipient`. ECDSA and FIDO (`sk-`) SSH keys are rejected with a message that says so. The comment part of an SSH key is kept in the stored string for the reader's benefit but ignored for cryptography. Deduplication compares the key material, not the comment.

Plugin recipients require the plugin binary `age-plugin-<name>` on `PATH`. If it is missing, the error names the binary.

#### 3.3.2 Identity sources

An identity is anything that can decrypt a recipients store. The CLI resolves identity files in this order and uses all of them together (age tries each):

1. `--identity PATH`, repeatable.
2. `TROUSSEAU_IDENTITY_FILE`, one path.
3. `identity.files` from the config file, in order.
4. `<config_dir>/trousseau/identity.txt` if it exists.
5. `~/.ssh/id_ed25519` then `~/.ssh/id_rsa` if they exist.

Each path may be:

- An age identity file (lines starting with `AGE-SECRET-KEY-1`, comments with `#`), parsed with `age::IdentityFile`.
- An age-encrypted identity file (armored age file whose payload is an identity file), handled with `age::encrypted::Identity`; the passphrase is requested through the `Callbacks` implementation.
- An OpenSSH private key (`-----BEGIN OPENSSH PRIVATE KEY-----`), parsed with `age::ssh::Identity`; an encrypted key triggers a passphrase prompt through `Callbacks`.
- An age plugin identity file (`AGE-PLUGIN-<NAME>-1...`) handled through `age::plugin::IdentityPluginV1`.

If no identity source exists at all, the error is `no identity found; run 'trousseau init' or pass --identity` with exit code 4.

#### 3.3.3 Passphrase sources for passphrase stores

1. `--passphrase-file PATH`: file content with one trailing newline stripped.
2. `TROUSSEAU_PASSPHRASE`: accepted, and when stdin is a terminal a warning is printed once on stderr: `warning: TROUSSEAU_PASSPHRASE is set; prefer --passphrase-file or an identity`.
3. Interactive hidden prompt. With `--no-input` or a non-terminal stdin and none of the above, fail with exit code 4.

On `init --passphrase` and `rekey --to-passphrase`, an interactive prompt asks twice and compares. Minimum passphrase length: 8 bytes. There is no maximum.

scrypt work factor: use the `age` crate default when sealing. When opening, cap with `with_max_work_factor(22)` so a hostile header cannot pin the CPU indefinitely.

### 3.4 Configuration file

`<config_dir>/trousseau/config.toml`, optional. Unknown keys are errors.

```toml
[identity]
# Additional identity files, tried in order after --identity and TROUSSEAU_IDENTITY_FILE.
files = ["~/.ssh/id_ed25519"]

[clipboard]
# Seconds before the clipboard is cleared after `get --clip`. 0 disables clearing.
timeout_seconds = 45

[run]
# Prefix prepended to every environment variable name in `run` and `env`.
env_prefix = ""

[migrate]
# Path or name of the gpg binary used to read legacy OpenPGP stores.
gpg = "gpg"
```

`~` at the start of a path expands to the home directory. Nothing else expands. `TROUSSEAU_CONFIG` overrides the config path.

### 3.5 Command-line interface

#### 3.5.1 Global behavior

- Binary name: `trousseau`. `--version` prints `trousseau <semver>`.
- Global flags, valid before or after the subcommand: `--store PATH`, `--global`, `--identity PATH` (repeatable), `--passphrase-file PATH`, `--json`, `--quiet`, `--no-input`.
- `--json`: read commands emit exactly one JSON document on stdout and nothing else on stdout. Write commands emit `{"ok": true, ...}` on success. Errors in `--json` mode go to stderr as `{"error": {"code": "<snake_case>", "message": "..."}}` and stdout stays empty.
- `--quiet`: suppress informational stderr lines. Errors still print.
- `--no-input`: never prompt. Anything that would have prompted fails with exit code 4 (secret input) or 2 (confirmation).
- stdin not a terminal implies `--no-input` for confirmations, but secret values are still read from stdin where the command says so.
- Colors: none. Do not add a color dependency.
- Panics: `human-panic` is configured with a report path; the report MUST NOT include environment variables or arguments. Verify with a test that sets `TROUSSEAU_PASSPHRASE` and triggers the hidden `__panic-test` subcommand (debug builds only, behind `#[cfg(debug_assertions)]`).

Exit codes:

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

Locking: read commands take a shared lock, write commands an exclusive lock, for the whole duration of the command. Lock wait is 5 seconds, then exit 6. Lock files live in the cache directory (3.2).

Timestamps: `updated_at` on the store and on the entry are set to now (UTC, seconds) on every mutation. `created_at` never changes.

#### 3.5.2 `init`

```
trousseau init [--recipient R]... [--recipients-file PATH] [--passphrase] [--no-self]
```

- Refuses if the target store exists: exit 8.
- `--passphrase`: creates a passphrase store. Mutually exclusive with `--recipient`, `--recipients-file`, `--no-self`.
- Otherwise creates a recipients store. Recipients are the union of `--recipient` values, lines of `--recipients-file` (blank lines and `#` comments ignored), and, unless `--no-self`, the user's own recipient.
- The user's own recipient comes from the default identity file. If that file does not exist, `init` generates an X25519 identity with `age::x25519::Identity::generate()`, writes it to the default identity path with mode 0600 (creating the directory 0700), and prints on stderr: `created identity <path>` and `your recipient: age1...`.
- If interactive and `~/.ssh/id_ed25519.pub` exists and is not already in the list, ask: `Also encrypt to your SSH key ~/.ssh/id_ed25519.pub? [y/N]`. Default no.
- With `--no-self` and no recipients: exit 2 with `at least one recipient is required`.
- Writes an empty store (`entries: {}`) and prints `created <path>` on stderr. In `--json` mode: `{"ok": true, "path": "...", "kind": "recipients", "recipients": [...]}`.
- Never touches `.gitignore`.

#### 3.5.3 `info`

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

`kind` comes from the age header. The rest needs decryption; if unlocking fails, print `path` and `kind` and the lines `schema: (locked)` etc., and exit 0. `--json`: `{"path", "kind", "schema", "recipients", "entries", "updated_at", "locked": bool}`.

#### 3.5.4 `set`

```
trousseau set KEY [--from-file PATH | --from-env NAME] [--binary] [--env NAME] [--description TEXT]
```

Value source, exactly one:

1. `--from-file PATH`: file bytes verbatim. `-` means stdin, verbatim, no newline stripping.
2. `--from-env NAME`: the named environment variable's value. Missing variable: exit 1.
3. Neither, stdin is a terminal: hidden prompt `Value for KEY: `. No confirmation.
4. Neither, stdin is not a terminal: read all of stdin, then strip exactly one trailing `\n` or `\r\n`.

Encoding: `base64` if `--binary` or if the bytes are not valid UTF-8 or contain NUL; otherwise `utf8`.

Existing key: keep `created_at`, keep `env` and `description` unless the flag is given; set `updated_at`. Empty values are allowed.

Output: `set KEY` on stderr unless `--quiet`. `--json`: `{"ok": true, "key": "...", "encoding": "utf8", "created": bool}`.

#### 3.5.5 `get`

```
trousseau get KEY [--out PATH] [--force] [--clip]
```

- Default: raw value bytes to stdout. If stdout is a terminal and encoding is `utf8`, append `\n`. If stdout is a terminal and encoding is `base64`, refuse with `binary value; use --out or pipe the output`, exit 1. If stdout is not a terminal, write raw bytes with no newline in either case.
- `--out PATH`: write bytes to the file with mode 0600. If the file exists, exit 8 unless `--force` is given.
- `--clip`: copy to the clipboard, print `copied KEY to clipboard, clearing in 45s` on stderr, and schedule clearing (3.5.16). Requires the `clipboard` feature; otherwise exit 1 with `built without clipboard support`.
- `--json`: `{"key", "value", "encoding", "env", "description", "created_at", "updated_at"}` where `value` is the stored representation (utf8 string or base64 string).
- Missing key: exit 5, message `key not found: KEY`.

#### 3.5.6 `ls`

```
trousseau ls [PREFIX] [--long]
```

- Lists keys sorted bytewise, one per line.
- `PREFIX` filters to keys equal to `PREFIX` or starting with `PREFIX/`. It is a path prefix, not a string prefix: `ls data` does not list `database/password`.
- `--long`: a table with columns `KEY`, `ENC`, `ENV`, `UPDATED`, `DESCRIPTION`. Never values.
- `--json`: array of `{"key", "encoding", "env", "description", "created_at", "updated_at"}`.
- An empty result is exit 0 with empty output.

#### 3.5.7 `rm`

```
trousseau rm KEY... [--force]
```

Removes each key. Any missing key: exit 5 and nothing is written, unless `--force`, which ignores missing keys. Output per removed key on stderr: `removed KEY`. `--json`: `{"ok": true, "removed": [...]}`.

#### 3.5.8 `mv`

```
trousseau mv OLD NEW [--force]
```

Renames, keeping all entry metadata and `created_at`. `OLD` missing: exit 5. `NEW` exists: exit 8 unless `--force`. `--json`: `{"ok": true, "from": "...", "to": "..."}`.

#### 3.5.9 `recipients`

```
trousseau recipients ls
trousseau recipients add R...
trousseau recipients rm R...
```

- Only valid on recipients stores; on a passphrase store: exit 1 with `this is a passphrase store; use 'rekey --to-recipients'`.
- `ls`: one recipient per line as stored. `--json`: array of strings.
- `add`: validates each (3.3.1), ignores duplicates with a stderr note, saves. Saving re-encrypts to the new set.
- `rm`: removes by exact match or by key material match (SSH comment differences do not matter). Refuses to remove the last recipient: exit 2. If the removed recipient corresponds to one of the caller's own identities, print `warning: you removed your own recipient; you will not be able to open this store after this command` and require confirmation (interactive) or `--force`.

#### 3.5.10 `rekey`

```
trousseau rekey [--to-passphrase | --to-recipients R... ]
```

- No flags: re-encrypt to the current recipient set (or passphrase) with a fresh file key. Useful after a suspected leak of the file.
- `--to-passphrase`: convert to a passphrase store; prompt twice; `recipients` becomes empty and `kind` becomes `passphrase`.
- `--to-recipients R...`: convert to a recipients store with exactly the listed recipients (the user's own recipient is not added implicitly).
- `--json`: `{"ok": true, "kind": "...", "recipients": [...]}`.

#### 3.5.11 `export` and `import`

```
trousseau export [--format json|dotenv|toml] [--out PATH]
trousseau import [--format json|dotenv|toml] [--strategy keep|overwrite|fail] [PATH]
```

- `json` (default): the full payload document (3.1.2). `import --format json` accepts the same document and merges its `entries`; it ignores `recipients`, `kind` and the store-level timestamps of the imported document. Imported entries keep their `created_at` and get `updated_at` = now.
- `dotenv`: one `NAME="value"` line per utf8 entry using the entry's resolved env name (3.1.4, no prefix); `"`, `\` and newline are escaped as `\"`, `\\`, `\n`. base64 entries are skipped with a stderr warning. `import --format dotenv` parses `NAME=value`, `NAME="value"` and `NAME='value'` lines, ignores blank and `#` lines, sets key = lowercase of NAME with `_` kept, and sets `env` = NAME.
- `toml`: the `edit` document format (3.5.12). Import parses the same.
- `--strategy` for `import`: `keep` (existing keys win), `overwrite` (imported wins), `fail` (any collision aborts with exit 8). Default `fail`.
- `export` writes to stdout; `--out PATH` writes a 0600 file and exits 8 if the file exists, unless `--force`.
- `export` always requires unlocking. Print a one-line stderr warning when exporting plaintext to a file: `warning: <path> contains plaintext secrets`.

#### 3.5.12 `edit`

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

- Exactly one of `value` (utf8) or `value_base64` per table. Tables keyed by the entry key.
- Scratch file: created with `tempfile::Builder` with prefix `trousseau-` and suffix `.toml`, mode 0600, in the first existing directory of: `$XDG_RUNTIME_DIR`, `/dev/shm`, the system temp dir. On Windows: the system temp dir.
- Editor: `$VISUAL`, else `$EDITOR`, else `vi` on Unix and `notepad` on Windows. Split with `shell-words`. Run with stdin, stdout, stderr inherited. Non-zero editor exit: abort, exit 1, scratch deleted.
- After the editor exits: if the file is empty or unchanged, print `no changes` and exit 0. Parse; on error print the TOML error with line number and, if interactive, ask `Reopen the editor? [Y/n]`; otherwise exit 1. Scratch is deleted in every path, including panics (use a guard with `Drop`).
- Apply: removed tables remove entries; changed values or metadata bump `updated_at`; unchanged entries keep their timestamps; new tables create entries with both timestamps now.
- Document in `docs/cli.md` how to stop common editors from leaving backup or swap files (vim: `set nobackup nowritebackup noswapfile` for `trousseau-*` files; VS Code: `code --wait` and the temp path).

#### 3.5.13 `run`

```
trousseau run [--env-prefix P] [--only KEYPREFIX]... [--no-inherit] -- CMD [ARGS]...
```

- Unlocks the store, builds the environment (3.1.4), and executes `CMD`.
- Selection: all entries, or only those under any `--only` path prefix (same semantics as `ls PREFIX`).
- base64 entries are skipped with one stderr warning per entry: `skipping binary entry KEY`.
- `--no-inherit`: the child gets only the injected variables plus `PATH`, `HOME`, `TMPDIR`, `TERM`, `LANG`, `LC_*` if set. Default: inherit the parent's environment with injected variables overriding.
- Env name conflicts: exit 8 before executing anything.
- Unix: replace the current process with `std::os::unix::process::CommandExt::exec`. If `exec` returns, print the error and exit 1. Windows: spawn, wait, exit with the child's code.
- Nothing is written to disk. No lock is held while the child runs: release the shared lock after decryption, before `exec`.
- `--json` is rejected for `run` (exit 2).

#### 3.5.14 `env`

```
trousseau env [--format shell|dotenv|json] [--env-prefix P] [--only KEYPREFIX]...
```

- `shell` (default): `export NAME='value'` lines with single-quote escaping (`'` becomes `'\''`). Safe for `eval "$(trousseau env)"` and for direnv's `.envrc`.
- `dotenv`: as in 3.5.11.
- `json`: object `{"NAME": "value", ...}`. `--json` is an alias for `--format json`.
- Same selection, skipping and conflict rules as `run`.

#### 3.5.15 `migrate`

```
trousseau migrate SOURCE [--gpg PATH] [--gnupg-home PATH] [--new-passphrase-file PATH]
```

- `SOURCE` is a v0.4 store file (usually `~/.trousseau`). The target is the store resolved by the normal rules; it MUST NOT already exist (exit 8). Recipients or passphrase for the target follow the `init` rules, so `migrate` accepts the same `--recipient`, `--recipients-file`, `--passphrase`, `--no-self` flags.
- Detects the legacy format (3.7). Not legacy: exit 1 with `not a v0.4 store`.
- AES stores: the legacy passphrase comes from `--passphrase-file`, `TROUSSEAU_PASSPHRASE`, or a hidden prompt, in that order. When the target is also a passphrase store, the new passphrase comes from `--new-passphrase-file` or a separate double prompt `New passphrase for the migrated store: `. The two passphrases are never assumed equal.
- OpenPGP stores: run the `gpg` binary (3.7.3). Not found: exit 1 naming the binary.
- Key sanitization: legacy keys may contain any characters. Each key is converted by replacing every character outside `[A-Za-z0-9._/-]` with `_`, collapsing repeated `/`, trimming leading and trailing `/`, and, if the result is empty or invalid, using `migrated/<index>`. Collisions get `_2`, `_3` suffixes. Every renamed key is printed: `renamed "easy as" -> easy_as`.
- Legacy `recipients` (PGP key ids) are printed for information and not carried.
- The source file is never modified or deleted.
- Output: `migrated N entries to <path>`. `--json`: `{"ok": true, "path", "entries": N, "renamed": [{"from","to"}], "legacy_recipients": [...]}`.

#### 3.5.16 Clipboard clearing

`get --clip` copies the value, then spawns a detached copy of itself: `trousseau __clip-clear <sha256-hex-of-value> <seconds>` (hidden subcommand, not shown in help). That process sleeps, reads the clipboard, and clears it only if the clipboard's sha256 still matches. On Unix, detach with `setsid`-like semantics via `Command::new(current_exe).process_group(0)` and redirected stdio to null. The value itself is never passed on the command line; only its hash is.

#### 3.5.17 `completions` and `man`

`trousseau completions <bash|zsh|fish|powershell|elvish>` prints a completion script from `clap_complete`. `trousseau man` prints the roff page for the top-level command; `trousseau man <subcommand>` for a subcommand. Both are used by the release pipeline to generate packaged files.

### 3.6 Error catalogue (library)

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("store not found at {path}")]
    StoreNotFound { path: PathBuf },
    #[error("legacy v0.4 store detected at {path}; run 'trousseau migrate'")]
    LegacyStore { path: PathBuf },
    #[error("invalid store: {reason}")]
    InvalidStore { reason: String },
    #[error("store schema {found} is newer than this build supports ({supported})")]
    SchemaTooNew { found: u32, supported: u32 },
    #[error("cannot unlock store: {reason}")]
    Unlock { reason: String },          // wrong passphrase, no matching identity, plugin failure
    #[error("no identity available")]
    NoIdentity,
    #[error("invalid recipient: {input}: {reason}")]
    InvalidRecipient { input: String, reason: String },
    #[error("invalid key: {input}: {reason}")]
    InvalidKey { input: String, reason: String },
    #[error("key not found: {key}")]
    KeyNotFound { key: String },
    #[error("key already exists: {key}")]
    KeyExists { key: String },
    #[error("environment name {name} is produced by both {a} and {b}")]
    EnvConflict { name: String, a: String, b: String },
    #[error("store is locked by another process")]
    LockTimeout,
    #[error("value too large: {bytes} bytes (limit {limit})")]
    TooLarge { bytes: usize, limit: usize },
    #[error("legacy store error: {reason}")]
    Legacy { reason: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Error messages never contain values, passphrases, or identity material. The CLI maps variants to exit codes in `exit.rs` with an exhaustive `match` (no wildcard arm).

### 3.7 Legacy v0.4 format

#### 3.7.1 Envelope

A JSON object with exactly these keys:

```json
{"crypto_type": 0, "crypto_algorithm": 1, "_data": "<base64>"}
```

- `crypto_algorithm`: `1` = AES-256-CFB (symmetric), `0` = OpenPGP. This field decides; `crypto_type` (0 symmetric, 1 asymmetric) is informational only, because the Go defaults code could leave it at 0 in some stores.
- `_data`: standard base64 of the ciphertext bytes.

Detection: parse as JSON object; all three keys present; `_data` decodes as base64. Anything else is not legacy.

#### 3.7.2 AES-256-CFB payload

`_data` decodes to `salt (16 bytes) || iv (16 bytes) || ciphertext`. Key = scrypt(passphrase, salt, N=65536 (log2 16), r=16, p=1, dkLen=32). Cipher: AES-256 in full-block CFB mode (CFB-128, the `cfb-mode` crate's default segment size), no padding, no MAC. Decrypt and then parse 3.7.4. A wrong passphrase produces garbage that fails JSON parsing; report it as `Error::Unlock { reason: "wrong passphrase or corrupted store" }`.

#### 3.7.3 OpenPGP payload

`_data` decodes to an ASCII-armored `-----BEGIN PGP MESSAGE-----` block. Decrypt by spawning: `<gpg> --batch --quiet --decrypt` with the armored bytes on stdin and, when `--gnupg-home` is given, the environment variable `GNUPGHOME` set to it. Capture stdout. Non-zero exit: `Error::Legacy { reason: "<last line of gpg stderr>" }`. Do not parse keyrings. Do not pass any passphrase; gpg-agent or pinentry handles it.

#### 3.7.4 Inner document

```json
{
  "metadata": {"version": "0.4.1", "created_at": "...", "last_modified_at": "...", "recipients": ["4B7D890"]},
  "data": {"abc": "123", "easy as": "do re mi"}
}
```

`data` is a flat string-to-string map. Unknown fields are ignored here (the legacy writer was not strict).

### 3.8 Threat model summary

Committed in full as `docs/threat-model.md` in step 1.2, copied from the plan page's section. The implementation-relevant consequences:

- Any recipient can rewrite the store undetectably by cryptography. Nothing in v1 signs.
- Secrets are zeroized on drop but memory is not locked.
- `run` cannot protect the child's environment.
- `edit` scratch files and clipboard managers are documented exposures.

---

## 4. Phases and steps

Each step lists: branch, goal, deliverables, instructions, tests, acceptance, review focus. Estimated size is changed lines including tests.

### Phase 0: preserve and clear

#### Step 0.0 (reviewer, no PR): freeze the Go code

The reviewer runs, on an up-to-date `master`:

```bash
git switch master && git pull
git branch legacy-go
git tag -a go-final -m "Final commit of the Go implementation before the Rust rewrite"
git branch rust-rewrite
git push origin legacy-go go-final rust-rewrite
```

Then in the GitHub repository settings: set `rust-rewrite` as a protected branch that requires a PR and passing checks, and disable squash merging for the repository for the duration of the rewrite (or instruct every merge to use "Create a merge commit").

#### Step 0.1: legacy fixtures

Branch `rr/0.1-legacy-fixtures`, base `rust-rewrite`. Size: small, mostly binary fixtures and one script.

Goal: capture real v0.4 store files, produced by the real Go binary, before the Go code is removed. The migrator in step 2.5 is tested against these.

Deliverables:

- `scripts/legacy/generate-fixtures.sh`, executable, POSIX sh or bash with `set -euo pipefail`.
- `crates/trousseau/tests/fixtures/legacy/README.md` describing every file, the passphrase, and how to regenerate.
- `crates/trousseau/tests/fixtures/legacy/symmetric-v0.4.json` (the AES store).
- `crates/trousseau/tests/fixtures/legacy/asymmetric-v0.4.json` (the OpenPGP store).
- `crates/trousseau/tests/fixtures/legacy/test-key.sec.asc` and `test-key.pub.asc` (throwaway RSA key pair, no passphrase).
- `crates/trousseau/tests/fixtures/legacy/expected.json`: the exact `data` map every fixture contains.

Instructions:

1. The script clones the repository at tag `go-final` into a temporary directory and builds it: `go build -o "$TMP/trousseau" ./cmd/trousseau`. It requires `go` and `gpg` on `PATH` and fails early if either is missing.
2. Symmetric fixture. With `TROUSSEAU_PASSPHRASE='correct horse battery staple'` and `--config "$TMP/config.toml" --store "$TMP/symmetric.json"`, run `create --encryption-type symmetric`, then `set abc 123`, `set 'easy as' 'do re mi'`, `set 'multi/line' "$(printf 'a\nb')"` (use `--file` with a file containing `a\nb` so the newline is exact), `set unicode 'héllo wörld'`. Copy the file to the fixture path.
3. Asymmetric fixture. Generate a throwaway key in `GNUPGHOME="$TMP/gnupg"` with `gpg --batch --gen-key` and this parameter file: `Key-Type: RSA`, `Key-Length: 2048`, `Subkey-Type: RSA`, `Subkey-Length: 2048`, `Name-Real: Trousseau Test`, `Name-Email: test@trousseau.invalid`, `Expire-Date: 0`, `Preferences: SHA256 SHA1 AES256 AES ZLIB ZIP Uncompressed`, `%no-protection`, `%commit`. Export `gpg --armor --export-secret-keys` and `gpg --armor --export` to the two `.asc` fixtures. The Go binary needs legacy keyring files, so write them: `gpg --export > "$TMP/gnupg/pubring.gpg"` and `gpg --export-secret-keys > "$TMP/gnupg/secring.gpg"`. First attempt: run the Go binary with `--gnupg-home "$TMP/gnupg" --store "$TMP/asymmetric.json" create test@trousseau.invalid`, then the same four `set` calls with `TROUSSEAU_PASSPHRASE=''`. If that succeeds, copy the file. If it fails (the deprecated Go OpenPGP library rejects some modern key preferences), fall back to constructing the file by hand, which is byte-for-byte what the Go code would produce: decrypt the symmetric fixture with the Go binary (`--store "$TMP/symmetric.json" export --plain`), encrypt that inner document with `gpg --armor --encrypt -r test@trousseau.invalid --trust-model always`, base64 the armored output, and write `{"crypto_type":1,"crypto_algorithm":0,"_data":"<base64>"}`. The fixture README states which path produced the committed file.
4. `expected.json` is written from the `export --plain` output's `data` object.
5. The script prints the fixture paths and exits 0.

Tests: none in this step (no Rust yet). The README's regeneration instructions are the test.

Acceptance: running the script on a machine with Go and GnuPG reproduces files that decrypt to `expected.json` (the reviewer checks the AES one by hand with the Go binary: `--store symmetric-v0.4.json export --plain`).

Review focus: the fixtures decrypt; the throwaway key is clearly marked as test material; no real key material is committed.

#### Step 0.2: remove the Go implementation

Branch `rr/0.2-remove-go`, base `rr/0.1-legacy-fixtures`. Size: large deletion, tiny addition.

Deliverables:

- Deleted: `cmd/`, `internal/`, `pkg/`, `go.mod`, `go.sum`, `.goreleaser.yml`, `.github/workflows/go.yml`, `.github/workflows/goreleaser.yml`, `.github/dependabot.yml`, `scripts/autocompletion/`, `scripts/vagrant_provision.sh`, `History.md`, `trousseau.gif`, `CONTRIBUTING.md`.
- Kept: `LICENSE`, `AUTHORS.md`, `CHANGELOG.md`, `scripts/legacy/`, `crates/trousseau/tests/fixtures/`.
- Replaced `README.md` with a 15-line stub: name, one sentence, "Rewrite in progress on the `rust-rewrite` branch; the Go implementation lives on the `legacy-go` branch and the `go-final` tag; the last Go release is v0.4.1", and a link to `docs/IMPLEMENTATION_PLAN.md`.
- Replaced `.gitignore` with: `/target`, `**/*.rs.bk`, `.DS_Store`.
- Added `docs/IMPLEMENTATION_PLAN.md` (this file) and `docs/design/rebuilding-trousseau.html` (the plan page, exported from the artifact).

Acceptance: `git ls-files` shows no `.go` file. The repository has no build system yet; CI is absent in this PR and that is expected. State it in the PR body.

Review focus: nothing kept that should go, nothing deleted that should stay.

### Phase 1: scaffold

#### Step 1.1: workspace and CI

Branch `rr/1.1-workspace`, base `rr/0.2-remove-go`. Size: ~250 lines.

Deliverables:

- `Cargo.toml` workspace with `members = ["crates/*"]`, `resolver = "3"`, `[workspace.package]` with `edition = "2024"`, `rust-version = "1.89"`, `license = "MIT"`, `repository`, `authors = ["Théo Crevon <theo@crevon.me>"]`, and `[workspace.dependencies]` listing every crate from 2.3 with versions and features. Also `[profile.release]` with `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"` is NOT set (human-panic needs unwinding).
- `crates/trousseau/Cargo.toml`: `name = "trousseau"`, `version = "1.0.0-alpha.1"`, `publish = false` for now, description, keywords, categories, the lint block from 2.4, dependencies from the workspace. `src/lib.rs` with crate docs and `pub mod error;` only; `error.rs` with the enum from 3.6.
- `crates/trousseau-cli/Cargo.toml`: `name = "trousseau-cli"`, `[[bin]] name = "trousseau"`, `publish = false`, features `default = ["clipboard"]`, `clipboard = ["dep:arboard"]`. `src/main.rs` that builds a clap `Command` named `trousseau` with `--version` and nothing else, and exits 0.
- `rust-toolchain.toml`, `rustfmt.toml` as in 2.4.
- `.github/workflows/ci.yml`: triggers on `pull_request` and on `push` to `rust-rewrite` and `master`. Jobs: `fmt` (ubuntu), `clippy` (ubuntu, `--all-targets --all-features -D warnings`), `test` (matrix ubuntu-latest, macos-latest, windows-latest; `cargo test --workspace --all-features`), `doc` (ubuntu, `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps`), `msrv` (ubuntu, toolchain `1.89`, `cargo check --workspace --all-features`). Pin every action to a commit SHA with a trailing `# vX.Y.Z` comment, as motus does. Use `actions-rust-lang/setup-rust-toolchain` and enable its cache. Set `permissions: contents: read` at the top.
- `justfile` with recipes `check` (fmt, clippy, test, doc in sequence), `test`, `lint`, `fmt`.

Tests: one unit test in `error.rs` asserting `Display` of `Error::KeyNotFound` equals `key not found: foo`. One integration test in `crates/trousseau-cli/tests/version.rs` using `assert_cmd` that `trousseau --version` prints `trousseau 1.0.0-alpha.1`.

Acceptance: `just check` passes locally; CI is green on all three OSes.

Review focus: workspace dependency table matches 2.3; lint block is exact; action pins are SHAs.

#### Step 1.2: specification documents

Branch `rr/1.2-docs`, base `rr/1.1-workspace`. Size: documentation only.

Deliverables: `docs/format.md` (from 3.1, 3.2, 3.3), `docs/cli.md` (from 3.5, formatted as a reference with one section per command), `docs/threat-model.md` (from the plan page), `docs/migration.md` (from 3.7 and 3.5.15, written for an end user). Each file starts with a one-paragraph summary and a "Status: implemented in step X" line updated by later steps.

Acceptance: the documents contain no statement that contradicts section 3. The reviewer reads them as the user-facing contract.

#### Step 1.3: project tooling

Branch `rr/1.3-tooling`, base `rr/1.2-docs`. Size: ~150 lines.

Deliverables:

- `deny.toml` copied from motus, license allow-list adjusted for MIT as the project license (keep the same allow-list of dependency licenses), `[advisories] version = 2`.
- `.github/workflows/security.yml`: daily `cargo deny check advisories` plus on PR `cargo deny check` (all sections). Pinned action SHAs.
- `renovate.json` copied from motus.
- `.github/PULL_REQUEST_TEMPLATE.md` from 1.6.
- `.github/CODEOWNERS` with `* @oleiade`.
- `SECURITY.md`: report to `theo@crevon.me`, expect acknowledgement within 7 days, supported versions table (empty until 1.0), no bug bounty.

Acceptance: `cargo deny check` passes; security workflow runs green on the PR.

### Phase 2: library

#### Step 2.1: schema

Branch `rr/2.1-schema`, base `rr/1.3-tooling`. Size: ~500 lines.

Deliverables in `crates/trousseau/src/schema.rs`:

```rust
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_VALUE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Key(String);
impl Key {
    pub fn parse(s: &str) -> Result<Self, Error>;
    pub fn as_str(&self) -> &str;
    pub fn has_path_prefix(&self, prefix: &Key) -> bool;   // equal or starts with prefix + "/"
    pub fn env_name(&self, prefix: &str) -> String;         // 3.1.4 without the explicit override
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding { Utf8, Base64 }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreKind { Recipients, Passphrase }

/// Secret bytes. `Debug` prints `Value(<redacted>)`. Zeroized on drop.
#[derive(Clone)]
pub struct Value(SecretBox<Vec<u8>>);
impl Value {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error>;   // enforces MAX_VALUE_BYTES
    pub fn expose(&self) -> &[u8];
    pub fn detect_encoding(&self) -> Encoding;                    // utf8 without NUL, else base64
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub value: Value,          // custom (de)serialization: string field per 3.1.2 plus `encoding`
    pub encoding: Encoding,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")] pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")] pub updated_at: OffsetDateTime,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    pub schema: u32,
    pub kind: StoreKind,
    #[serde(with = "time::serde::rfc3339")] pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")] pub updated_at: OffsetDateTime,
    pub recipients: Vec<String>,
    pub entries: BTreeMap<Key, Entry>,
}
impl Store {
    pub fn new(kind: StoreKind, recipients: Vec<String>, now: OffsetDateTime) -> Self;
    pub fn validate(&self) -> Result<(), Error>;                 // every rule in 3.1.2
    pub fn to_json(&self) -> Result<Vec<u8>, Error>;             // pretty, sorted, trailing newline, size check
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error>;       // size check first, then schema check, then validate
    pub fn set(&mut self, key: Key, value: Value, env: Option<String>, description: Option<String>, now: OffsetDateTime) -> bool; // returns created
    pub fn remove(&mut self, key: &Key, now: OffsetDateTime) -> Option<Entry>;
    pub fn rename(&mut self, from: &Key, to: Key, force: bool, now: OffsetDateTime) -> Result<(), Error>;
    pub fn env_map(&self, prefix: &str, only: &[Key]) -> Result<BTreeMap<String, &Entry>, Error>; // conflicts -> EnvConflict; base64 entries excluded
}
```

Implementation notes:

- `Entry` serialization: write a manual `Serialize`/`Deserialize` or use a private mirror struct with `value: String`. The stored string is the raw utf8 text or the base64 text depending on `encoding`. On deserialize, decode base64 for `Base64` and validate utf8 for `Utf8`; mismatch is `InvalidStore`.
- Timestamps are truncated to whole seconds before storing (`replace_nanosecond(0)`).
- `Debug` for `Value`, `Entry`, `Store` MUST redact values. Implement `Debug` by hand for `Value`; the derived `Debug` on `Entry` then prints the redacted form.
- `validate` checks: schema == 1, kind/recipients consistency, recipients sorted and unique (string compare here; key-material dedup happens in `identity.rs`), env override regex, description length, encoding/value consistency, total size.

Tests (`crates/trousseau/src/schema.rs` unit tests and `crates/trousseau/tests/schema_roundtrip.rs`):

- Key: accepts `a`, `a/b`, `a.b-c_d/1`, rejects ``, `/a`, `a/`, `a//b`, `a/../b`, `.hidden`, `-x`, a 257-byte key, a key with a space, a key with `é`.
- `has_path_prefix`: `database/password` has prefix `database`, not `data`; equal keys match.
- `env_name`: `database/password` → `DATABASE_PASSWORD`; `1abc` → `_1ABC`; `a.b-c` → `A_B_C`; with prefix `APP_` → `APP_DATABASE_PASSWORD`.
- Round trip: `proptest` strategy generating stores with random valid keys, utf8 and binary values, optional env/description; `from_json(to_json(s)) == s` (implement `PartialEq` on `Store` for tests only, comparing exposed bytes).
- Golden: the JSON in 3.1.2 parses, validates, and re-serializes byte-identically (commit it as `tests/fixtures/schema1.json`).
- `deny_unknown_fields`: a payload with an extra field fails with `InvalidStore`.
- `schema: 2` fails with `SchemaTooNew`.
- `kind: passphrase` with a recipient fails; `kind: recipients` with none fails.
- `env_map`: conflict between `a-b` and `a_b` (both `A_B`) is `EnvConflict`; explicit `env` on one resolves it; base64 entries are excluded.
- `Debug` of a store containing `hunter2` does not contain `hunter2`.

Acceptance: `cargo test -p trousseau` passes; `cargo doc` has no warnings (every public item documented).

Review focus: the validation rules match 3.1 exactly; `Debug` redaction; no `unwrap`.

#### Step 2.2: envelope

Branch `rr/2.2-envelope`, base `rr/2.1-schema`. Size: ~400 lines.

Deliverables in `crates/trousseau/src/envelope.rs`:

```rust
pub enum EnvelopeKind { Recipients, Passphrase }

/// Inspect the age header without decrypting.
pub fn peek_kind(armored: &[u8]) -> Result<EnvelopeKind, Error>;   // InvalidStore if not armored age

pub fn seal_to_recipients(plaintext: &[u8], recipients: &[Box<dyn age::Recipient + Send>]) -> Result<Vec<u8>, Error>;
pub fn seal_with_passphrase(plaintext: &[u8], passphrase: &SecretString) -> Result<Vec<u8>, Error>;
pub fn open_with_identities(armored: &[u8], identities: &[Box<dyn age::Identity>]) -> Result<Zeroizing<Vec<u8>>, Error>;
pub fn open_with_passphrase(armored: &[u8], passphrase: &SecretString) -> Result<Zeroizing<Vec<u8>>, Error>;
```

Implementation notes:

- Output of `seal_*` is ASCII armored (`age::armor::ArmoredWriter` with `Format::AsciiArmor`), ending with a newline.
- `peek_kind`: wrap in `ArmoredReader`, construct the `age::Decryptor`, and ask whether it is scrypt-based (the 0.11 API exposes this on the decryptor; verify the exact method name on docs.rs). Anything that fails to parse is `InvalidStore { reason: "not an age file" }`.
- `open_with_passphrase` uses `age::scrypt::Identity` with `with_max_work_factor(22)`. `seal_with_passphrase` uses `age::scrypt::Recipient` with the crate default work factor.
- Map age's `DecryptError::NoMatchingKeys` and `DecryptError::DecryptionFailed` to `Error::Unlock` with reasons `"no matching identity"` and `"wrong passphrase or corrupted store"`. Map `ExcessiveWork` to `Unlock { reason: "passphrase work factor too high" }`.
- Read the whole decrypted stream into a `Zeroizing<Vec<u8>>`, enforcing `MAX_PAYLOAD_BYTES` with `take()` and failing with `TooLarge` if more bytes remain.

Tests (`crates/trousseau/tests/envelope.rs`):

- Round trip with one generated X25519 identity.
- Round trip with two X25519 recipients; each identity alone opens it.
- Round trip with an SSH ed25519 recipient: generate a key pair in the test with the `age::ssh` types? The crate cannot generate SSH keys; commit a throwaway test key pair `tests/fixtures/ssh/id_ed25519` and `.pub` generated once with `ssh-keygen -t ed25519 -N ''` and clearly labeled. Same for `id_rsa` (2048 bits). Both without passphrase, and one extra `id_ed25519_pw` with passphrase `test` for the callback test in step 2.3.
- Passphrase round trip; wrong passphrase is `Unlock`; `peek_kind` returns `Passphrase`.
- `peek_kind` on a recipients file returns `Recipients`; on `b"hello"` returns `InvalidStore`; on an unarmored binary age file returns `InvalidStore`.
- Tampering: flip one byte in the armored body (not in the header lines); opening fails with `Unlock`.
- Interop check, `#[ignore]` unless `rage` or `age` is on `PATH`: seal a payload, run `age -d -i <identity file>` on it, compare bytes. CI installs `age` on ubuntu in step 4.x; until then the test is skipped, and the PR body says so.

Acceptance: tests pass on all three OSes (SSH fixtures included).

Review focus: error mapping, size cap, work factor cap, no plaintext in errors.

#### Step 2.3: recipients and identities

Branch `rr/2.3-identity`, base `rr/2.2-envelope`. Size: ~450 lines.

Deliverables in `crates/trousseau/src/identity.rs`:

```rust
pub enum RecipientKind { X25519, SshEd25519, SshRsa, Plugin(String) }

pub struct ParsedRecipient { pub kind: RecipientKind, pub canonical: String, pub original: String }
pub fn parse_recipient(input: &str) -> Result<ParsedRecipient, Error>;
/// Build age recipients for sealing. Plugin recipients need `callbacks` for the plugin protocol.
pub fn to_age_recipients(list: &[String], callbacks: impl age::Callbacks) -> Result<Vec<Box<dyn age::Recipient + Send>>, Error>;
/// Dedup by key material, keep first occurrence's original string, sort by canonical form.
pub fn normalize_recipients(list: Vec<String>) -> Result<Vec<String>, Error>;
pub fn same_recipient(a: &str, b: &str) -> bool;

pub enum IdentityFileKind { AgePlain, AgeEncrypted, Ssh, Plugin }
pub fn detect_identity_file(bytes: &[u8]) -> Option<IdentityFileKind>;
pub fn load_identities(paths: &[PathBuf], callbacks: impl age::Callbacks + Clone) -> Result<Vec<Box<dyn age::Identity>>, Error>;
pub struct GeneratedIdentity { pub identity_file_contents: SecretString, pub recipient: String }
pub fn generate_identity(now: OffsetDateTime) -> GeneratedIdentity;   // file contents: "# created: <rfc3339>\n# public key: age1...\nAGE-SECRET-KEY-1...\n"
/// Recipient strings corresponding to the given identities (for `recipients rm` self-check and `init`).
pub fn own_recipients(paths: &[PathBuf]) -> Vec<String>;             // best effort: plain age identities and unencrypted SSH keys only
```

Implementation notes:

- `canonical` for SSH keys is `"<type> <base64>"` without the comment. For X25519 it is the Bech32 string lowercased. For plugins, the string as-is.
- `load_identities` reads each file, detects the kind, and delegates: `age::IdentityFile::from_buffer` then `into_identities` (plain and plugin), `age::encrypted::Identity::from_buffer` (encrypted), `age::ssh::Identity::from_buffer` (SSH) with `with_callbacks` for passphrase prompting. A file that matches no kind is `Error::InvalidStore`-like; add a variant `InvalidIdentity { path, reason }` to `error.rs` in this step.
- The library defines no prompting. The `Callbacks` implementor lives in the CLI. Tests use a small struct returning fixed answers.

Tests:

- Parse: the age recipient in 3.1.2 (any valid one), the SSH ed25519 and rsa fixtures' public keys, `age1yubikey1abc` (kind Plugin("yubikey")), and rejections for `ecdsa-sha2-nistp256 ...`, `sk-ssh-ed25519@openssh.com ...`, `AGE-SECRET-KEY-1...` (an identity, not a recipient), `hello`.
- `normalize_recipients`: same SSH key with two comments collapses to one; result sorted; invalid entry errors.
- `load_identities`: plain age file with two identities and a comment; SSH ed25519 unencrypted; SSH ed25519 encrypted with callbacks returning `test`; encrypted age identity file (create one in the test with `seal_with_passphrase` from 2.2 over a generated identity) with callbacks; garbage file fails with `InvalidIdentity`.
- `generate_identity` output parses back through `load_identities` and its recipient opens what it sealed.

Acceptance: tests pass on all three OSes. Plugin loading is exercised only by a parse test; no plugin binary in CI.

Review focus: dedup logic, the list of rejected SSH key types, no secret in `Debug` (the `GeneratedIdentity` struct needs a manual `Debug`).

#### Step 2.4: store I/O

Branch `rr/2.4-store-io`, base `rr/2.3-identity`. Size: ~450 lines.

Deliverables in `crates/trousseau/src/store.rs`:

```rust
pub const PROJECT_STORE_FILENAME: &str = ".trousseau";

pub enum Resolved { Explicit(PathBuf), Project(PathBuf), Personal(PathBuf) }
impl Resolved { pub fn path(&self) -> &Path; }

pub struct Locator<'a> { pub explicit: Option<PathBuf>, pub env: Option<PathBuf>, pub global: bool, pub cwd: &'a Path, pub personal: &'a Path }
impl Locator<'_> {
    pub fn resolve(&self) -> Resolved;             // 3.2 order; for open
    pub fn resolve_for_init(&self) -> Resolved;    // same, but rule 4 = cwd join ".trousseau"
}
pub fn find_project_store(start: &Path) -> Option<PathBuf>;

pub struct LockGuard(/* fd_lock guard + File */);
pub enum LockMode { Shared, Exclusive }
pub fn lock(store_path: &Path, lock_dir: &Path, mode: LockMode, timeout: Duration) -> Result<LockGuard, Error>;

pub enum RawStore { Current(Vec<u8>), Legacy(Vec<u8>) }
/// Reads the file; classifies as current (armored age), legacy (3.7.1) or invalid.
pub fn read_raw(path: &Path) -> Result<RawStore, Error>;   // StoreNotFound, LegacyStore, InvalidStore

pub enum Unlock<'a> { Identities(&'a [Box<dyn age::Identity>]), Passphrase(&'a SecretString) }
pub fn open(path: &Path, unlock: Unlock<'_>) -> Result<Store, Error>;   // read_raw + envelope::open_* + Store::from_json

pub enum Seal<'a> { Recipients(&'a [Box<dyn age::Recipient + Send>]), Passphrase(&'a SecretString) }
pub fn save(path: &Path, store: &Store, seal: Seal<'_>) -> Result<(), Error>;   // validate, to_json, seal, write_atomic

pub fn write_atomic(path: &Path, bytes: &[u8], mode: u32) -> Result<(), Error>;
```

Implementation notes:

- `write_atomic`: `tempfile::Builder::new().prefix(".trousseau-").tempfile_in(parent)`, set permissions to `mode` on Unix before writing, write all, `sync_all`, `persist` over the target (rename), then open the parent directory and `sync_all` it on Unix. On Windows, `persist` fails if the target exists and is open elsewhere; retry three times with 50 ms sleeps, then fail.
- `lock` opens or creates `<lock_dir>/<sha256(canonical path)>.lock`, creating `lock_dir` with 0700, and polls `try_read`/`try_write` every 50 ms until `timeout`, then `LockTimeout`. The `LockGuard` holds the file open.
- `read_raw` order: not found → `StoreNotFound`; starts with `-----BEGIN AGE ENCRYPTED FILE-----` → `Current`; parses as legacy per 3.7.1 → `Legacy`; else `InvalidStore`.
- `open` consumes `Legacy` as `Error::LegacyStore`.
- `find_project_store` walks up with `Path::ancestors`, returns the first `.trousseau` that is a regular file. Symlinks to regular files count.

Tests (`crates/trousseau/tests/store_io.rs`, using `tempfile::tempdir`):

- Locator precedence: every rule in 3.2 with a table-driven test; `resolve_for_init` does not walk up.
- `find_project_store` from a nested directory; none found at root of the temp dir.
- `write_atomic` leaves no temp file behind on success; on failure to persist (make the parent read-only on Unix, skip on Windows) the original file is untouched.
- Mode on Unix is 0600 after write (check `metadata().permissions().mode() & 0o777`).
- Lock: two guards `Shared` coexist; `Exclusive` with an existing `Shared` times out with `LockTimeout` at a 200 ms timeout; after dropping, it succeeds.
- `read_raw` classification for: an age file, the legacy symmetric fixture, a JSON file missing `_data`, an empty file, a random binary file.
- `open`/`save` round trip with identities and with passphrase; `save` then `open` returns an equal store; the saved file starts with the armor header; saving twice produces different ciphertexts (fresh file key) for the same store.

Acceptance: tests pass on all three OSes (permission-based tests gated with `#[cfg(unix)]`).

Review focus: atomicity sequence, lock semantics, the legacy detection being conservative.

#### Step 2.5: legacy reader

Branch `rr/2.5-legacy`, base `rr/2.4-store-io`. Size: ~350 lines.

Deliverables in `crates/trousseau/src/legacy.rs`:

```rust
pub enum LegacyAlgorithm { Aes256Cfb, OpenPgp }
pub struct LegacyEnvelope { pub algorithm: LegacyAlgorithm, pub data: Vec<u8> }
pub fn parse_envelope(bytes: &[u8]) -> Result<LegacyEnvelope, Error>;   // 3.7.1

pub struct LegacyStore { pub version: Option<String>, pub recipients: Vec<String>, pub data: BTreeMap<String, Value> }
pub fn decrypt_aes(env: &LegacyEnvelope, passphrase: &SecretString) -> Result<LegacyStore, Error>;
pub struct GpgOptions { pub binary: PathBuf, pub gnupg_home: Option<PathBuf> }
pub fn decrypt_gpg(env: &LegacyEnvelope, opts: &GpgOptions) -> Result<LegacyStore, Error>;

pub struct Conversion { pub store: Store, pub renamed: Vec<(String, Key)>, pub legacy_recipients: Vec<String> }
pub fn convert(legacy: LegacyStore, kind: StoreKind, recipients: Vec<String>, now: OffsetDateTime) -> Conversion;  // 3.5.15 sanitization
```

Implementation notes:

- `decrypt_aes` follows 3.7.2 precisely. Use `scrypt::scrypt(passphrase, salt, &Params::new(16, 16, 1, 32)?, &mut key)`. Decrypt with `cfb_mode::Decryptor::<aes::Aes256>::new(&key.into(), &iv.into())` and `decrypt` in place on a copy. Zeroize the key and plaintext buffers.
- `decrypt_gpg` spawns the process with `Stdio::piped()` for all three streams, writes the armored bytes to stdin in a thread or before waiting (the message is small; write then close stdin, then `wait_with_output`). Set `GNUPGHOME` only if given; never clear the environment (gpg-agent needs it). Include the last non-empty stderr line in the error.
- Inner document parsing (3.7.4) is shared by both.

Tests (`crates/trousseau/tests/legacy.rs`):

- `parse_envelope` on both fixtures succeeds with the right algorithm; on the schema1 fixture and on garbage returns an error.
- `decrypt_aes` on `symmetric-v0.4.json` with the README passphrase yields exactly `expected.json`; with a wrong passphrase returns `Unlock`.
- `decrypt_gpg` on `asymmetric-v0.4.json`: gated with a runtime check that `gpg` is on `PATH`, otherwise the test prints `skipped: gpg not found` and returns. When it runs: create a temp `GNUPGHOME`, `gpg --batch --import test-key.sec.asc`, then decrypt and compare with `expected.json`. Ubuntu runners have gpg; the macOS and Windows jobs will skip.
- `convert`: keys `abc`, `easy as`, `multi/line`, `unicode` become `abc`, `easy_as`, `multi/line`, `unicode`; a legacy key `//weird//` becomes `weird`; empty key becomes `migrated/0`; two keys that sanitize to the same name get `_2`; timestamps are `now`; store validates.

Acceptance: the AES fixture decrypts in CI on all OSes; the gpg test runs on ubuntu.

Review focus: exact KDF parameters, that the gpg invocation passes nothing secret on argv, sanitization rules.

### Phase 3: command line

Every CLI step adds integration tests in `crates/trousseau-cli/tests/` using `assert_cmd`, with `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` (and `APPDATA`, `LOCALAPPDATA` on Windows) pointed at a temp directory, and a helper `tests/common/mod.rs` that creates a store with a known identity. Never rely on the developer's real home directory.

#### Step 3.1: CLI skeleton, config, context, output

Branch `rr/3.1-cli-skeleton`, base `rr/2.5-legacy`. Size: ~550 lines.

Deliverables:

- `cli.rs`: the full clap tree from 3.5 with every subcommand and flag declared now, so `--help` is complete from this step on. Subcommands not yet implemented return `anyhow!("not implemented yet (step N)")` with exit 1. Global flags per 3.5.1 with `env = "TROUSSEAU_STORE"` on `--store` and `env = "TROUSSEAU_IDENTITY_FILE"` on a hidden single-value counterpart merged into the identity list in `context.rs`.
- `config.rs`: `Config` struct mirroring 3.4 with `#[serde(deny_unknown_fields)]`, defaults, `~` expansion, `Config::load(path: Option<&Path>) -> anyhow::Result<Config>` where a missing file yields defaults and a malformed one is an error.
- `context.rs`: `Context` built once in `main`: resolved directories (etcetera), config, `Locator`, identity paths in 3.3.2 order (existing files only), passphrase source, output mode (`Human | Json`), `quiet`, `no_input`, `is_stdin_tty`, `is_stdout_tty`. Methods: `unlock(&self, path) -> Result<Store>` implementing the `peek_kind` → prompt-or-load logic, `seal_for(&self, store) -> Result<Seal>` building recipients or asking the passphrase (cached in the context for the command's duration as a `SecretString`), `lock(path, mode)`.
- `prompt.rs`: `hidden(prompt) -> Result<SecretString>`, `hidden_confirm(prompt) -> Result<SecretString>` (asks twice, min 8 bytes), `confirm(question, default) -> Result<bool>`, and `struct CliCallbacks` implementing `age::Callbacks` (display to stderr, `request_passphrase` via `hidden`, `confirm` via `confirm`, `request_public_string` via a visible stdin line). All respect `no_input`.
- `output.rs`: `info(&ctx, msg)` (stderr unless quiet), `warn(&ctx, msg)` (stderr always), `json(&ctx, value)` (stdout, one document, trailing newline), `raw(bytes)` (stdout), `table(rows)` (simple column alignment, no dependency).
- `exit.rs`: `fn code_for(err: &anyhow::Error) -> i32` downcasting to `trousseau::Error` per 3.5.1, and `fn report(err, ctx)` printing either the human message or the JSON error object.
- `main.rs`: install `human-panic` with metadata that omits arguments and environment (write a custom `panic` hook that calls human-panic's report with a message only; verify by test), parse, build context, dispatch, map errors to exit codes.

Tests:

- `--help` lists every subcommand from 3.5; snapshot with `insta`.
- `TROUSSEAU_CONFIG` pointing at a config with an unknown key exits 1 and names the key.
- `--json` with an unimplemented subcommand prints a JSON error object on stderr and nothing on stdout.
- The panic test: debug-only hidden subcommand `__panic-test`, run with `TROUSSEAU_PASSPHRASE=hunter2` and an argument `hunter3`; assert the report file and stderr contain neither string.
- Exit code mapping unit test covering every `Error` variant.

Acceptance: `trousseau --help` and `trousseau <cmd> --help` for every command match `docs/cli.md`.

Review focus: `context.rs` precedence logic vs 3.2 and 3.3; nothing prints outside `output.rs`.

#### Step 3.2: `init` and `info`

Branch `rr/3.2-init-info`, base `rr/3.1-cli-skeleton`. Size: ~400 lines.

Implements 3.5.2 and 3.5.3.

Tests:

- `init` in an empty temp project creates `.trousseau` (armored), creates the default identity with mode 0600 on Unix, prints the recipient on stderr; `info` reports `kind: recipients`, `entries: 0`.
- `init` twice exits 8.
- `init --passphrase` with `--passphrase-file`; `info` reports `kind: passphrase`.
- `init --recipient <ssh pub fixture> --no-self`; `info --json` reports one recipient; unlocking with the SSH fixture identity works (`info` with `--identity`).
- `init --no-self` without recipients exits 2.
- `init --global` writes to the data dir path, not the cwd.
- `init --no-input` with an existing `~/.ssh/id_ed25519.pub` in the fake home does not add it (assert recipients count).
- `info` on a locked store (no identity available) exits 0 with `locked: true` in JSON.

Review focus: identity generation and file modes; the SSH offer only when interactive.

#### Step 3.3: `set`, `get`, `ls`, `rm`, `mv`

Branch `rr/3.3-crud`, base `rr/3.2-init-info`. Size: ~600 lines. Split into `3.3a` (set, get) and `3.3b` (ls, rm, mv) if over budget.

Implements 3.5.4 through 3.5.8. `--clip` is declared but returns `not implemented yet (step 3.9)` until then.

Tests:

- `set k` with piped stdin `v\n` stores `v`; `get k` piped prints `v` with no newline; `get k --json` shows `encoding: utf8`.
- `set k --from-file` with bytes `\x00\x01\xff` stores base64; `get k` piped returns the exact bytes. The terminal refusal is not testable portably, so test the `--out` path instead: the file is created 0600 with the exact bytes; a second `--out` on the same path exits 8; with `--force` it succeeds.
- `set k --from-env NAME`; missing NAME exits 1.
- `set` existing key preserves `created_at` and `description`, bumps `updated_at` (compare JSON from `get --json` before and after with a 1 s sleep or by injecting a fake clock through an env var `TROUSSEAU_TEST_NOW` honored only in debug builds).
- `ls` sorted; `ls database` path-prefix semantics; `ls --long` never contains a value; `ls --json` snapshot.
- `rm` missing exits 5 and leaves the store unchanged; `rm --force` ignores; `rm a b` removes both atomically.
- `mv` to an existing key exits 8; `--force` overwrites; metadata carried.
- `--no-input` with `set k` and a terminal-less stdin still reads stdin (piped) and succeeds.
- Lock: start `set` with stdin held open (do not write), then run `ls` concurrently: `ls` waits and then succeeds after the first finishes; run a second `set` with a 200 ms timeout override (`TROUSSEAU_TEST_LOCK_TIMEOUT_MS`, debug only) and assert exit 6.
- Every write leaves exactly one `.trousseau` file and no temp file in the directory.

Review focus: newline handling rules, encoding detection, exit codes.

#### Step 3.4: `recipients` and `rekey`

Branch `rr/3.4-recipients`, base `rr/3.3-crud`. Size: ~350 lines.

Implements 3.5.9 and 3.5.10.

Tests:

- `recipients add <ssh pub>`; the SSH identity can now open the store; `recipients ls` shows two.
- Adding a duplicate with a different comment: still two, stderr note.
- `recipients rm` down to zero exits 2.
- `recipients rm <own>` with `--no-input` and without `--force` exits 2; with `--force` succeeds and the own identity can no longer open (exit 4).
- `rekey` alone changes the ciphertext and the store still opens.
- `rekey --to-passphrase` then `info` reports passphrase; `recipients ls` exits 1 with the hint; `rekey --to-recipients <age recipient>` converts back.
- On a passphrase store, `recipients add` exits 1.

Review focus: the "removing yourself" guard; that `rekey` never writes before both unlock and reseal succeed.

#### Step 3.5: `export` and `import`

Branch `rr/3.5-export-import`, base `rr/3.4-recipients`. Size: ~450 lines.

Implements 3.5.11.

Tests:

- `export` (json) of a store round-trips through `import --strategy overwrite` into a fresh store: values, encodings, env and description equal; `created_at` preserved from the document; `updated_at` set to now.
- `export --format dotenv` escapes `"`, `\` and newlines; a base64 entry is skipped with a warning.
- `import --format dotenv` from a file with comments, blank lines, single and double quotes, unquoted values, and `export NAME=value` lines (accept the `export ` prefix).
- `import` default strategy `fail` exits 8 on collision and writes nothing; `keep` leaves existing; `overwrite` replaces.
- `export --out` refuses to overwrite without `--force` (exit 8) and warns about plaintext.
- `export --format toml` output parses with the `edit` parser from step 3.7 (add the cross-test in 3.7).

Review focus: dotenv escaping both ways; no partial writes on `fail`.

#### Step 3.6: `run` and `env`

Branch `rr/3.6-run-env`, base `rr/3.5-export-import`. Size: ~400 lines.

Implements 3.5.13 and 3.5.14.

Tests:

- `run -- sh -c 'printf %s "$DATABASE_PASSWORD"'` prints the value (on Windows use `cmd /C echo %DATABASE_PASSWORD%` and trim).
- Exit code passthrough: `run -- sh -c 'exit 7'` exits 7.
- `--env-prefix APP_`; `--only database` excludes other keys; `--no-inherit` drops a variable set in the test's environment but keeps `PATH`.
- Conflict exits 8 and does not execute (use a command that would create a file; assert the file is absent).
- Binary entry skipped with a warning on stderr.
- `env` shell format: a value containing `'` and a newline round-trips through `sh -c 'eval "$(cat env.txt)"; printf %s "$K"'`.
- `env --format json` snapshot; `env --json` equivalent.
- No lock is held during the child: start `run -- sh -c 'sleep 2'` in the background and run `set` concurrently with `TROUSSEAU_TEST_LOCK_TIMEOUT_MS=200`; the `set` succeeds.

Review focus: `exec` usage, environment construction, that the lock is released before `exec`.

#### Step 3.7: `edit`

Branch `rr/3.7-edit`, base `rr/3.6-run-env`. Size: ~450 lines.

Implements 3.5.12. The editor is invoked through `$VISUAL`/`$EDITOR`; tests set `EDITOR` to a small script committed under `crates/trousseau-cli/tests/editors/` (a shell script on Unix, a `.cmd` on Windows) that rewrites the scratch file according to an env var `TEST_EDIT_ACTION`: `noop`, `empty`, `add`, `remove`, `change`, `corrupt`, `fail`.

Tests:

- `noop` prints `no changes` and does not rewrite the store (ciphertext identical).
- `empty` aborts, store identical.
- `add` creates an entry with both timestamps now; `remove` deletes; `change` bumps `updated_at` only for the changed key.
- `corrupt` with `--no-input` exits 1 and the store is unchanged; scratch file is gone (assert the scratch directory has no `trousseau-*.toml`).
- `fail` (editor exits 3) aborts with exit 1.
- Scratch file mode is 0600 on Unix (the editor script records the mode into a side file).
- The document produced by `export --format toml` parses and applies unchanged (`noop` equivalence).

Review focus: scratch file lifecycle and directory choice, the `Drop` guard, no secrets left behind on any path.

#### Step 3.8: `migrate`

Branch `rr/3.8-migrate`, base `rr/3.7-edit`. Size: ~350 lines.

Implements 3.5.15 over `legacy.rs`.

Tests:

- Migrate the symmetric fixture into a new recipients store with `--passphrase-file` holding the legacy passphrase and `--recipient <age recipient>` `--no-self`; `ls` shows `abc`, `easy_as`, `multi/line`, `unicode`; `get multi/line` bytes equal `a\nb`; stderr contains `renamed "easy as" -> easy_as`.
- Migrate into a passphrase store with two different passphrases: `--passphrase-file` for the legacy one, `--new-passphrase-file` for the new one; the new store opens only with the new passphrase.
- Target exists: exit 8, source untouched (compare hashes).
- Not a legacy file: exit 1.
- OpenPGP fixture: gated on `gpg` presence like step 2.5; with `--gnupg-home` pointing at a temp home where the test key was imported.
- Opening a legacy file with any other command exits 7 with the `migrate` hint.

Review focus: sanitization output, source never written, both passphrases handled distinctly.

#### Step 3.9: clipboard, completions, man pages

Branch `rr/3.9-clip-completions`, base `rr/3.8-migrate`. Size: ~350 lines.

Implements 3.5.16 and 3.5.17, and enables `get --clip`.

Tests:

- `completions bash|zsh|fish|powershell|elvish` each print non-empty output containing `trousseau`.
- `man` prints roff containing `.TH`; `man get` contains `--clip`.
- `get --clip` cannot be tested reliably in headless CI. Unit test the hash comparison and the argument parsing of `__clip-clear`, and mark one end-to-end clipboard test `#[ignore]` with a note that it runs locally only.
- Building with `--no-default-features` compiles, and `get --clip` exits 1 with the "built without clipboard support" message.

Review focus: the detached process never receives the value; the feature gate.

### Phase 4: release engineering

#### Step 4.1: release pipeline

Branch `rr/4.1-release`, base `rr/3.9-clip-completions`. Size: ~250 lines of YAML and config.

Deliverables:

- `.goreleaser.yaml` modeled on motus: builds via `cargo zigbuild` for `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` (build the Windows target natively in a separate job if zigbuild cannot; document which). Archives `tar.gz` (zip on Windows) containing the binary, `LICENSE`, `README.md`, generated completions and man page (produced by running the built binary in the pipeline). `nfpms` for `deb` and `apk` with the same metadata style as motus. `brews` for `oleiade/homebrew-tap`. Checksums `sha256`. `draft: true`.
- `.github/workflows/release.yml` triggered on tags `v*`, mirroring motus: install toolchains and zig, run GoReleaser. Permissions: `contents: write`, `id-token: write`, `attestations: write`.
- `Cross.toml` only if needed by the chosen targets.
- `Makefile` or `justfile` recipes `release-dry-run` that runs `goreleaser release --snapshot --clean`.

Acceptance: `goreleaser release --snapshot --clean` succeeds locally (the agent runs it and pastes the artifact list in the PR), and a pre-release tag `v1.0.0-alpha.1` pushed by the reviewer after merge produces a draft release with all archives.

#### Step 4.2: provenance, SBOM, security workflow

Branch `rr/4.2-provenance`, base `rr/4.1-release`. Size: ~100 lines.

Deliverables:

- In `release.yml`: `actions/attest-build-provenance` (pinned SHA) over `dist/*.tar.gz`, `dist/*.zip`, `dist/*.deb`, `dist/*.apk`, and the checksums file.
- SBOM: install `cargo-cyclonedx`, run `cargo cyclonedx --format json --all`, upload `trousseau.cdx.json` as a release asset via GoReleaser `extra_files`.
- `cargo auditable`: attempt `cargo auditable zigbuild`; if it does not compose, leave a comment in the workflow explaining and skip. Do not spend more than one iteration on it.
- `docs/cli.md` gains a "Verify a download" section showing `gh attestation verify <file> --owner oleiade` and `sha256sum -c`.

Acceptance: the next pre-release tag shows attestations on the release page and the SBOM asset.

#### Step 4.3: distribution channels and install docs

Branch `rr/4.3-distribution`, base `rr/4.2-provenance`. Size: documentation and small config.

Deliverables:

- apt repository publishing step identical to motus (whatever motus's release workflow does to push to `oleiade.github.io/deb`; copy it and adjust the package name).
- Homebrew formula generation verified against the tap.
- `cargo install trousseau-cli` path: set `publish = true` on the CLI crate only if the reviewer decides to publish to crates.io; otherwise document `cargo install --git`.
- README install section: Homebrew, apt, GitHub release archives with verification, `cargo install --git`, Windows zip.

Acceptance: reviewer installs the pre-release on macOS via the tap and on a Debian container via apt.

### Phase 5: documentation and 1.0

#### Step 5.1: README and docs pass

Branch `rr/5.1-readme`, base `rr/4.3-distribution`.

Deliverables:

- `README.md` structure, in order: name and one-sentence pitch; a 12-line quick start (`init`, `set`, `run`, commit the file, a teammate `recipients add`); what it defends against and what it does not (six bullets total, from `docs/threat-model.md`); install; the git workflow (commit `.trousseau`, add teammates, rotate with `rekey`); CI usage with an identity from a secret; migration from v0.4 in four lines; links to `docs/`; license.
- Every `docs/*.md` file's "Status" line updated to "implemented".
- `CONTRIBUTING.md` rewritten: toolchain, `just check`, PR expectations, the lint policy, how to add a command (touch `cli.rs`, `commands/`, `docs/cli.md`, tests).
- `CHANGELOG.md`: new top section `## 1.0.0-rc.1` summarizing the rewrite and listing removed features and the migration path, above the preserved legacy entries.

Acceptance: a reader can go from nothing to `trousseau run` in the quick start without opening `docs/`.

#### Step 5.2: release candidate

Branch `rr/5.2-rc1`, base `rr/5.1-readme`.

- Bump both crates to `1.0.0-rc.1`. Update the `--version` test.
- Reviewer tags `v1.0.0-rc.1` after merge. The release is published as a pre-release.
- Open the external review (reviewer's action, outside the repository). Scope: `envelope.rs`, `identity.rs`, `store.rs`, `context.rs`, `prompt.rs`, `edit.rs`, `run.rs`, `clip.rs`, the release pipeline.

#### Step 5.3: review findings

One branch per finding group, `rr/5.3-<slug>`, stacked in the order the findings are addressed. Each PR references the finding identifier. Findings the reviewer accepts without a change are recorded in `docs/threat-model.md` under "Accepted findings".

#### Step 5.4: 1.0.0 and merge to master

Branch `rr/5.4-1.0.0`, base: the last 5.3 branch.

- Bump to `1.0.0`. `CHANGELOG.md` section renamed. `SECURITY.md` supported versions: `1.x`.
- After merge into `rust-rewrite`, the reviewer opens the final PR `rust-rewrite` → `master`, merges with a merge commit, tags `v1.0.0` on `master`, and sets `master` as the default branch again if it was changed.

---

## 5. Appendices

### 5.1 JSON output shapes

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
| `env` | `{"NAME":"value"}` |
| `migrate` | `{"ok":true,"path":"...","entries":N,"renamed":[{"from":"...","to":"..."}],"legacy_recipients":["..."]}` |
| errors | stderr: `{"error":{"code":"key_not_found","message":"key not found: x"}}` |

Error `code` values are the `Error` variant names in snake_case, plus `usage`, `refused`, `child_failed`.

### 5.2 Test-only environment variables

Honored only in debug builds (`#[cfg(debug_assertions)]`), rejected otherwise:

| Variable | Effect |
|---|---|
| `TROUSSEAU_TEST_NOW` | RFC 3339 timestamp used as "now". |
| `TROUSSEAU_TEST_LOCK_TIMEOUT_MS` | Overrides the 5 s lock timeout. |

### 5.3 Checklist the reviewer applies to every PR

- The PR implements exactly one step and nothing from a later step.
- New code has tests in the same PR, and the tests exercise the acceptance criteria.
- No secret can reach argv, logs, errors, `Debug`, or a file outside the specification.
- No new dependency outside section 2.3, or it is justified in the PR body.
- `docs/cli.md` or `docs/format.md` were updated if behavior visible to users changed.
- The PR body follows the template, including test evidence.

### 5.4 Things the agent should not do

- Do not "improve" the age format usage with custom headers, extra MACs, or compression.
- Do not add a `--verbose` flag, colors, spinners, or a TUI.
- Do not add network code of any kind.
- Do not store anything in the OS keychain.
- Do not write to `master`, `legacy-go`, or any tag.
- Do not squash, force-push to, or rebase `rust-rewrite`.
- Do not modify `crates/trousseau/tests/fixtures/legacy/*` after step 0.1 without a new fixture generation run documented in the fixture README.
