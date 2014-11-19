#!/usr/bin/env bash

# Name of the entry storing the test keys passphrase in the keyring
TROUSSEAU_KEYRING_SERVICE_NAME=trousseau_test

# Test keys passphrase
TROUSSEAU_TEST_KEY_PASSPHRASE=trousseau

# setup_keyring_entry will create an entry for the trousseau test keys
# passphrase in the system keyring:
#   - OSX: entry will be created in the system keychain
#   - Linux: entry will whether be created in the gnome-keychain
#     or in your SecretService provider, according to your setup
setup_keyring_entry() {
    platform=$(uname)

    if [[ $platform == 'Linux' ]]; then
        platform='linux'
    elif [[ $platform == 'Darwin' ]]; then
        polite_sudo security add-generic-password -a "${USER}" -s "${TROUSSEAU_KEYRING_SERVICE_NAME}" -w "${TROUSSEAU_TEST_KEY_PASSPHRASE}" >&2
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}

# teardown_keyring_entry will remove the trousseau test keys passphrase
# entry from your system keyring.
teardown_keyring_entry() {
    platform=$(uname)

    if [[ $platform == 'Linux' ]]; then
        platform='linux'
    elif [[ $platform == 'Darwin' ]]; then
        polite_sudo security delete-generic-password -a "${USER}" -s "${TROUSSEAU_KEYRING_SERVICE_NAME}" >&2
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}
