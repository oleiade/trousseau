# Store format

A trousseau store is a single age-encrypted file. This document describes the
on-disk envelope, the JSON payload inside it, the key grammar, how a store is
found on disk, and how recipients and identities are parsed. It restates
`docs/IMPLEMENTATION_PLAN.md` section 3 (3.1 to 3.3); that document is the
source of truth and this one must not contradict it.

Status: implemented in steps 2.1 to 2.4.

## Envelope

A store is one file. Its bytes are an age file in ASCII armor: it begins with
`-----BEGIN AGE ENCRYPTED FILE-----` and ends with
`-----END AGE ENCRYPTED FILE-----` followed by a newline. Binary (unarmored)
age files are not accepted as stores.

Two kinds of store exist, distinguished by the age header:

- **Recipients store**: one or more age recipient stanzas (X25519,
  ssh-ed25519, ssh-rsa, or plugin). The scrypt stanza must not be present.
- **Passphrase store**: exactly one scrypt stanza.

age itself forbids mixing an scrypt stanza with other stanzas. The library
exposes `envelope::peek_kind` so the CLI can decide whether to prompt for a
passphrase or load identities before decrypting.

Every save produces a fresh age file key. There is no in-place update: a
store is always rewritten whole.

## Payload

The decrypted payload is UTF-8 JSON, one object, serialized with
`serde_json::to_vec_pretty`, keys in sorted order (`BTreeMap` everywhere),
with a trailing newline. Pretty printing is deliberate: an emergency
`age -d` on the store must produce something a human can read.

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
| `schema` | integer | Must be `1`. A reader that sees a larger value must fail with `Error::SchemaTooNew { found }`. A smaller value cannot occur in schema 1. |
| `kind` | `"recipients"` or `"passphrase"` | `recipients` requires the `recipients` array to be non-empty. `passphrase` requires it to be empty. Any other combination is `Error::InvalidStore`. |
| `created_at`, `updated_at` | string | RFC 3339, UTC, second precision, `Z` suffix. |
| `recipients` | array of string | Each string is a valid recipient per "Recipient strings" below, deduplicated, sorted. |
| `entries` | object | Keys satisfy "Keys" below. Sorted. |
| `entries.*.value` | string | For `utf8`: the value verbatim. For `base64`: standard base64 with padding (RFC 4648 section 4). |
| `entries.*.encoding` | `"utf8"` or `"base64"` | `utf8` requires the value to be valid UTF-8 without NUL bytes. Anything else is `base64`. |
| `entries.*.env` | string, optional | Overrides the derived environment variable name. Must match `^[A-Za-z_][A-Za-z0-9_]*$`. |
| `entries.*.description` | string, optional | Free text, max 1024 bytes. |
| `entries.*.created_at`, `updated_at` | string | As above. |

Unknown fields at any level are rejected (`#[serde(deny_unknown_fields)]`).
This is how schema mistakes surface early. Schema 2, if it ever exists,
changes the `schema` number.

Size limits: the payload is at most 16 MiB; a single value is at most
4 MiB. Exceeding either limit is `Error::TooLarge`.

## Keys

A key is a path of one or more segments separated by `/`.

- Segment grammar: `[A-Za-z0-9][A-Za-z0-9._-]*`.
- No empty segments, no leading or trailing `/`, no `.` or `..` segments.
- Total length at most 256 bytes. Case-sensitive. `Database/Password` and
  `database/password` are different keys.
- A key is never a prefix-parent of another key in a way that matters:
  `database` and `database/password` may both exist.

The `Key` newtype validates on construction and is the only way to build a
key.

## Environment variable mapping

`env_name(key, prefix)`:

1. Take the key string.
2. Replace every `/`, `-` and `.` with `_`.
3. Uppercase ASCII letters.
4. If the first character is a digit, prepend `_`.
5. Prepend `prefix` verbatim if non-empty (the prefix is validated with the
   same regex as `env`).

An entry with an explicit `env` uses it unchanged, and the prefix is still
prepended. Two entries that resolve to the same name are a conflict; `run`
and `env` fail with `Error::EnvConflict { name, keys }` listing both keys.

## Store discovery and paths

Directory roots come from `etcetera` with the XDG strategy on Linux and
macOS (`~/.config`, `~/.local/share`, `~/.cache`), and the Windows strategy
on Windows (`%APPDATA%`, `%LOCALAPPDATA%`). The macOS choice of XDG over
`~/Library` is deliberate: dotfile users and CI runners expect it, and it
matches what the age and sops ecosystems do.

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
4. The nearest `.trousseau` file walking up from the current directory to
   the filesystem root. Directories are checked, not files: `.trousseau`
   must be a regular file to match.
5. The personal store.

`init` uses the same order to decide where to create, except that rule 4
becomes "the current directory" (`init` never walks up). There is no
verbose flag: `info` reports the resolved path, and every error message
that concerns a store names its path.

File modes on Unix: store `0600`, identity `0600`, config `0600`,
directories `0700`. On Windows, trousseau relies on the user profile ACLs
and does not attempt ACL manipulation.

## Recipients and identities

### Recipient strings

Accepted forms, detected by prefix:

| Form | Example | Parsed with |
|---|---|---|
| age X25519 | `age1...` (Bech32, 62 chars) | `age::x25519::Recipient::from_str` |
| SSH public key | `ssh-ed25519 AAAA... comment` or `ssh-rsa AAAA... comment` | `age::ssh::Recipient::from_str` |
| Plugin | `age1<plugin>1...`, for example `age1yubikey1...` | `age::plugin::RecipientPluginV1` |

Anything else is `Error::InvalidRecipient`. ECDSA and FIDO (`sk-`) SSH keys
are rejected with a message that says so. The comment part of an SSH key is
kept in the stored string for the reader's benefit but ignored for
cryptography. Deduplication compares the key material, not the comment.

Plugin recipients require the plugin binary `age-plugin-<name>` on `PATH`.
If it is missing, the error names the binary.

### Identity sources

An identity is anything that can decrypt a recipients store. The CLI
resolves identity files in this order and uses all of them together (age
tries each):

1. `--identity PATH`, repeatable.
2. `TROUSSEAU_IDENTITY_FILE`, one path.
3. `identity.files` from the config file, in order.
4. `<config_dir>/trousseau/identity.txt` if it exists.
5. `~/.ssh/id_ed25519` then `~/.ssh/id_rsa` if they exist.

Each path may be:

- An age identity file (lines starting with `AGE-SECRET-KEY-1`, comments
  with `#`), parsed with `age::IdentityFile`.
- An age-encrypted identity file (armored age file whose payload is an
  identity file), handled with `age::encrypted::Identity`; the passphrase
  is requested through the `Callbacks` implementation.
- An OpenSSH private key (`-----BEGIN OPENSSH PRIVATE KEY-----`), parsed
  with `age::ssh::Identity`; an encrypted key triggers a passphrase prompt
  through `Callbacks`.
- An age plugin identity file (`AGE-PLUGIN-<NAME>-1...`) handled through
  `age::plugin::IdentityPluginV1`.

If no identity source exists at all, the error is
`no identity found; run 'trousseau init' or pass --identity` with exit
code 4.

### Passphrase sources for passphrase stores

1. `--passphrase-file PATH`: file content with one trailing newline
   stripped.
2. `TROUSSEAU_PASSPHRASE`: accepted, and when stdin is a terminal a warning
   is printed once on stderr: `warning: TROUSSEAU_PASSPHRASE is set; prefer
   --passphrase-file or an identity`.
3. Interactive hidden prompt. With `--no-input` or a non-terminal stdin and
   none of the above, fail with exit code 4.

On `init --passphrase` and `rekey --to-passphrase`, an interactive prompt
asks twice and compares. Minimum passphrase length: 8 bytes. There is no
maximum.

scrypt work factor: the `age` crate default is used when sealing. When
opening, it is capped with `with_max_work_factor(22)` so a hostile header
cannot pin the CPU indefinitely.

## Configuration file

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

`~` at the start of a path expands to the home directory. Nothing else
expands. `TROUSSEAU_CONFIG` overrides the config path.
