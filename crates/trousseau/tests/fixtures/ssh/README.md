# SSH test key fixtures

These are throwaway SSH key pairs generated solely for `trousseau`'s test
suite. They protect nothing, are committed to a public repository, and
MUST NOT be reused for anything else.

Generated once with:

```sh
ssh-keygen -t ed25519 -N '' -C 'trousseau-test' -f id_ed25519
ssh-keygen -t rsa -b 2048 -N '' -C 'trousseau-test' -f id_rsa
ssh-keygen -t ed25519 -N 'test' -C 'trousseau-test' -f id_ed25519_pw
```

| File | Type | Passphrase |
|---|---|---|
| `id_ed25519` / `id_ed25519.pub` | ed25519 | none |
| `id_rsa` / `id_rsa.pub` | RSA 2048 | none |
| `id_ed25519_pw` / `id_ed25519_pw.pub` | ed25519 | `test` |
