#!/usr/bin/env bash

polite_sudo() {
    sudo -p "Bats testing framework requires sudo access to setup the test key passphrase in keychain. Password: " "$@"
}
