#!/usr/bin/env bash

# Testing context
TMP_DIR=/tmp
TROUSSEAU_TEST_FILES_PREFIX=trousseau_test_
TROUSSEAU_TEST_FILES_WILDCARD="${TROUSSEAU_TEST_FILES_PREFIX}*"
TROUSSEAU_TEST_FILES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_WILDCARD}"

# Build context
TROUSSEAU_BINARY_DIR=../bin
TROUSSEAU_BINARY=../bin/trousseau

# Trousseau context
TROUSSEAU_TEST_KEY_ID=6F7FEB2D
TROUSSEAU_TEST_KEY_EMAIL=theo@trousseau.io
TROUSSEAU_TEST_STORE="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}store"
TROUSSEAU_TEST_STORE_CREATE="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}create_store"

# Keyring context
TROUSSEAU_KEYRING_SERVICE_NAME=trousseau_test
TROUSSEAU_TEST_KEY_PASSPHRASE=trousseau


polite_sudo() {
    sudo -p "Bats testing framework requires sudo access to setup the test key passphrase in keychain. Password: " "$@"
}

create_keyring_service() {
    platform=$(uname)

    if [[ $platform == 'Linux' ]]; then
        platform='linux'
    elif [[ $platform == 'Darwin' ]]; then
        polite_sudo security add-generic-password -a "${USER}" -s "${TROUSSEAU_KEYRING_SERVICE_NAME}" -w "${TROUSSEAU_TEST_KEY_PASSPHRASE}" &> /dev/null
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}

drop_keyring_service() {
    platform=$(uname)

    if [[ $platform == 'Linux' ]]; then
        platform='linux'
    elif [[ $platform == 'Darwin' ]]; then
        polite_sudo security delete-generic-password -a "${USER}" -s "${TROUSSEAU_KEYRING_SERVICE_NAME}" &> /dev/null
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}

# Setup and teardown
setup() {
    # Make sure to fail fast if trousseau was not built
    # and no binary path could be found
    if [ ! -d $TROUSSEAU_BINARY_DIR ]; then
        echo "trousseau binary dir not found ${TROUSSEAU_BINARY_DIR}"
        exit 1
    fi

    # Make sure to fail fast if trousseau was not built
    # and no binary could be found
    if [ ! -f $TROUSSEAU_BINARY ]; then
        echo "trousseau binary not found: ${TROUSSEAU_BINARY}"
        exit 1
    fi

    # Otherwise, create the base test store
    $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE create $TROUSSEAU_TEST_KEY_ID > /dev/null

    create_keyring_service
}

teardown() {
    # Remove every trousseau test prefixed files from 
    # tmp dir
    rm -rf $TROUSSEAU_TEST_FILES

    drop_keyring_service
}
