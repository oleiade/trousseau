#!/usr/bin/env bash


# polite_sudo exposes a verbose and explicit sudo command
# so any test who might need to ask for sudo password
# would be more user-friendly.
polite_sudo() {
    sudo -p "Bats testing framework requires sudo access to setup the test key passphrase in keychain. Password: " "$@"
}
