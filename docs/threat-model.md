# Threat model

This document says what trousseau defends against and what it does not, so
that using it is an informed choice rather than an assumption. It restates
the threat model from the design page (`docs/design/rebuilding-trousseau.html`)
and `docs/IMPLEMENTATION_PLAN.md` section 3.8.

Status: normative.

## Defends against

- Disclosure of the store at rest: a stolen laptop, a leaked bucket, a
  public repository, a synced folder.
- Undetected modification of the store by a non-recipient.
- Secrets in shell history, process lists, and crash logs via argv.
  trousseau never accepts a secret value as a command-line argument.
- Downgrade of KDF cost, because cost is bound in the age header.
- Partial writes and concurrent writers corrupting the store. Every save
  writes a sibling temp file, fsyncs it, and renames it over the store
  under an advisory lock.

## Does not defend against

- A compromised host while an identity usable on that host exists. If an
  attacker can run code as you, they can use whatever identity you can
  use.
- A malicious recipient rewriting the store. Any recipient can re-encrypt
  the store to a different content without any other recipient noticing
  cryptographically; v1 has no signatures. git history is the audit trail
  for a store kept in a repository. Signatures are the first post-1.0
  candidate.
- The child process of `run` leaking its own environment. Once secrets are
  injected as environment variables, trousseau has no further control over
  what the child does with them.
- Secrets paged to swap. trousseau zeroizes secret values on drop, but it
  does not lock memory, so a page containing a secret can still be written
  to swap by the operating system.
- Editors that leave backup or swap files during `edit`. This is
  documented, with the specific flags to disable it for vim and VS Code,
  in `docs/cli.md`.
- Clipboard managers that keep history. `get --clip` clears the system
  clipboard after a timeout, but it cannot reach into a third-party
  clipboard manager's own history.

## Implementation consequences

- Any recipient can rewrite the store undetectably by cryptography.
  Nothing in v1 signs the store.
- Secrets are zeroized on drop but memory is not locked.
- `run` cannot protect the child's environment once it has been injected.
- `edit` scratch files and clipboard managers are documented exposures,
  not solved problems.

## Accepted findings

Findings from the step 5.2 external review that the reviewer accepted
without a code change, recorded here per step 5.3, with the reasoning.
Empty until step 5.3 lands.
