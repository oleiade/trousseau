#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Testing context
TMP_DIR=tmp
TROUSSEAU_TEST_FILES_PREFIX=trousseau_test_
TROUSSEAU_TEST_FILES_WILDCARD="${TROUSSEAU_TEST_FILES_PREFIX}*"
TROUSSEAU_TEST_FILES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_WILDCARD}"

# Test gnupg context
TROUSSEAU_TEST_GNUPG_HOME="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}gnupg"
TROUSSEAU_TEST_KEYS_DIR="${DIR}/keys"

TROUSSEAU_TEST_FIRST_PUBLIC_KEY_FILE="${TROUSSEAU_TEST_KEYS_DIR}/trousseau_public_first_test.key"
TROUSSEAU_TEST_FIRST_PRIVATE_KEY_FILE="${TROUSSEAU_TEST_KEYS_DIR}/trousseau_private_first_test.key"
TROUSSEAU_TEST_FIRST_KEY_ID=6F7FEB2D
TROUSSEAU_TEST_FIRST_KEY_EMAIL=theo@trousseau.io

TROUSSEAU_TEST_SECOND_PUBLIC_KEY_FILE="${TROUSSEAU_TEST_KEYS_DIR}/trousseau_public_second_test.key"
TROUSSEAU_TEST_SECOND_PRIVATE_KEY_FILE="${TROUSSEAU_TEST_KEYS_DIR}/trousseau_private_second_test.key"
TROUSSEAU_TEST_SECOND_KEY_ID=EA7F9C59
TROUSSEAU_TEST_SECOND_KEY_EMAIL=theo@trousseau.io
#

# Keyring context
TROUSSEAU_KEYRING_SERVICE_NAME=trousseau_test
TROUSSEAU_TEST_KEY_PASSPHRASE=trousseau

# Build context
TROUSSEAU_BINARY_DIR="$DIR/../bin"
TROUSSEAU_TEST_OPTIONS="--gnupg-home=$TROUSSEAU_TEST_GNUPG_HOME"
TROUSSEAU_COMMAND="$TROUSSEAU_BINARY_DIR/trousseau"

# Trousseau global context
TROUSSEAU_TEST_STORE="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}store"
TROUSSEAU_TEST_STORE_AES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}aes_store"
TROUSSEAU_TEST_STORE_CREATE="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}create_store"
TROUSSEAU_TEST_STORE_CREATE_AES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}aes_create_store"

# setup_gpg creates a temporary gnupg keyring
# and import trousseau gpg test keys into it
setup_ggp() {
    # Create a temporary gnupg keyring for testing
    mkdir -m 0700 $TROUSSEAU_TEST_GNUPG_HOME

    # Import test keys in the test keyring
    gpg --quiet --homedir $TROUSSEAU_TEST_GNUPG_HOME --import $TROUSSEAU_TEST_FIRST_PUBLIC_KEY_FILE >&2
    gpg --quiet --homedir $TROUSSEAU_TEST_GNUPG_HOME --allow-secret-key-import --import $TROUSSEAU_TEST_FIRST_PRIVATE_KEY_FILE >&2 
    gpg --quiet --homedir $TROUSSEAU_TEST_GNUPG_HOME --import $TROUSSEAU_TEST_SECOND_PUBLIC_KEY_FILE >&2 
    gpg --quiet --homedir $TROUSSEAU_TEST_GNUPG_HOME --allow-secret-key-import --import $TROUSSEAU_TEST_SECOND_PRIVATE_KEY_FILE >&2 
}

# teardown_gpg destroys any existing temporary gnupg keyring
teardown_gpg() {
    rm -rf $TROUSSEAU_TEST_GNUPG_HOME
}

# setup_env exports tests common environment variables
setup_env() {
    export TROUSSEAU_PASSPHRASE=$TROUSSEAU_TEST_KEY_PASSPHRASE
}

# teardown_env cleans the environment from tests variables
teardown_env() {
    unset TROUSSEAU_PASSPHRASE
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

    setup_ggp
    setup_env

    # Otherwise, create the base test store
    $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME \
                       --store $TROUSSEAU_TEST_STORE \
                       create $TROUSSEAU_TEST_FIRST_KEY_ID > /dev/null

    $TROUSSEAU_COMMAND --store $TROUSSEAU_TEST_STORE_AES \
                       create --encryption-type 'symmetric' > /dev/null
}

teardown() {
    teardown_gpg
    teardown_env

    # Remove every trousseau test prefixed files from 
    # tmp dir
    rm -rf $TROUSSEAU_TEST_FILES
}
