# Migrating from the Go version (v0.4)

If you have a `~/.trousseau` file created by the old Go binary, this is how
you move its contents into a new, age-encrypted store. It restates
`docs/IMPLEMENTATION_PLAN.md` sections 3.5.15 and 3.7; that document is the
source of truth and this one must not contradict it.

Status: implemented.

## Before you start

- Your old store is never modified or deleted by this process. `migrate`
  only reads it.
- You need a new store to migrate into, and that store must not exist yet.
  If you already ran `trousseau init` at the target location, either
  remove that empty store or point `migrate` somewhere else with
  `--store`.
- Decide up front whether the new store should use age recipients (your
  own key, teammates' keys, a YubiKey) or a shared passphrase. This is the
  same choice `trousseau init` asks you to make; see `docs/cli.md`.

## Running the migration

```sh
trousseau migrate ~/.trousseau
```

By default this creates a recipients store using your own identity, the
same way `trousseau init` would. You can pass the same flags `init`
accepts:

```sh
# migrate into a store encrypted to a teammate's key too
trousseau migrate ~/.trousseau --recipient age1qz...

# migrate into a passphrase-protected store instead
trousseau migrate ~/.trousseau --passphrase
```

On success you will see:

```
migrated 14 entries to /home/t/.local/share/trousseau/default.trousseau
```

Add `--json` to any command for machine-readable output; `migrate`'s shape
is documented in `docs/cli.md`.

## What migrate detects

The old Go binary wrote one of two envelope kinds, both JSON objects with a
`crypto_type`, `crypto_algorithm` and `_data` key. `migrate` looks for
exactly that shape. If your file does not match, `migrate` fails with
`not a v0.4 store` and exit code 1: it is not going to guess.

## If your old store used a passphrase (AES)

`crypto_algorithm: 1` in the old file means the store was encrypted
symmetrically with a passphrase, using scrypt and AES-256-CFB.

`migrate` needs that old passphrase to read the store, in this order:

1. `--passphrase-file PATH`
2. the `TROUSSEAU_PASSPHRASE` environment variable
3. a hidden interactive prompt

If the new store is also passphrase-protected, `migrate` needs a second,
new passphrase for it. This is never assumed to be the same as the old
one. Supply it with `--new-passphrase-file PATH`, or answer the separate
prompt `New passphrase for the migrated store: ` twice.

A wrong old passphrase is reported as a decryption failure; there is no
way to tell "wrong passphrase" apart from "corrupted file" at this layer.

## If your old store used OpenPGP (GPG)

`crypto_algorithm: 0` means the store was encrypted to one or more OpenPGP
keys. `migrate` does not reimplement OpenPGP: it shells out to your local
`gpg` binary to do the decryption, the same way the old Go binary
effectively relied on your GnuPG setup.

- Make sure `gpg` is on your `PATH`, or point at it explicitly with
  `--gpg PATH`.
- If your keys live in a non-default GnuPG home directory, pass
  `--gnupg-home PATH`.
- `gpg-agent` and `pinentry` handle any passphrase prompt for your OpenPGP
  key; `migrate` never asks for or passes an OpenPGP passphrase itself.
- If `gpg` is not found, `migrate` fails and names the binary it looked
  for.

The old store's recipients were PGP key ids. Those are not age recipients,
so they cannot be carried into the new store automatically. `migrate`
prints them for your reference, then encrypts the migrated store using the
recipients (or passphrase) you asked for on the command line, exactly as
`trousseau init` would.

## Key names

The old format allowed almost any string as a key, including spaces and
punctuation that trousseau's new key grammar does not accept (see
`docs/format.md`). `migrate` rewrites each key:

1. Every character outside `[A-Za-z0-9._/-]` becomes `_`.
2. Repeated `/` are collapsed to one.
3. Leading and trailing `/` are trimmed.
4. If the result is empty or still not a valid key, it becomes
   `migrated/<index>`.
5. If two old keys land on the same new key, later ones get `_2`, `_3`,
   and so on appended.

Every rename is printed so you can see exactly what changed, for example:

```
renamed "easy as" -> easy_as
```

Check these lines. If a rename looks wrong for how you use a key (for
example, in a script that reads it by name), rename the key again with
`trousseau mv` after the migration.

## After migrating

- The new store lives wherever the normal store discovery rules put it
  (see `docs/format.md`), unless you passed `--store`.
- Your old `~/.trousseau` file is left exactly as it was. Once you have
  confirmed the new store looks right with `trousseau ls --long` and a
  couple of `trousseau get` checks, you can delete the old file yourself
  when you are ready; `migrate` will not do it for you.
- Update any scripts or CI configuration that referenced the old Go
  binary or the old key names to use the renamed keys and the new
  `trousseau` commands documented in `docs/cli.md`.
