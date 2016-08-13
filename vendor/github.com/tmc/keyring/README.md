# keyring provides cross-platform keychain access

http://godoc.org/github.com/tmc/keyring

Keyring provides a common interface to keyring/keychain tools.

License: ISC

Currently implemented:
- OSX
- SecretService
- gnome-keychain (via "gnome_keyring" build flag)

Contributions welcome!

Usage example:

```go
  err := keyring.Set("libraryFoo", "jack", "sacrifice")
  password, err := keyring.Get("libraryFoo", "jack")
  fmt.Println(password)
  Output: sacrifice
```

Example program:
```sh
 $ go get -v github.com/tmc/keyring/keyring-example && keyring-example
```


## Linux

Linux requirements:

### SecretService provider

- dbus

### gnome-keychain provider

- gnome-keychain headers
- Ubuntu/Debian: `libgnome-keyring-dev`
- Archlinux: `libgnome-keyring`

Tests on Linux:
```sh
 $ go test github.com/tmc/keyring
 $ # for gnome-keyring provider
 $ go test -tags gnome_keyring github.com/tmc/keyring
```

Example:
```sh
 $ # for SecretService provider
 $ go get -v github.com/tmc/keyring/keyring-example && keyring-example
 $ # for gnome-keyring provider
 $ go get -v -tags gnome_keyring github.com/tmc/keyring/keyring-example && keyring-example
```

