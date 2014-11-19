#!/usr/bin/env bash

# Working directory of the script
DIR="${BASH_SOURCE%/*}"
if [[ ! -d "$DIR" ]]; then DIR="$PWD"; fi

# Testing context
TMP_DIR=/tmp
TROUSSEAU_TEST_FILES_PREFIX=trousseau_test_
TROUSSEAU_TEST_FILES_WILDCARD="${TROUSSEAU_TEST_FILES_PREFIX}*"
TROUSSEAU_TEST_FILES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_WILDCARD}"

# Build context
TROUSSEAU_BINARY_DIR="$DIR/../bin"
TROUSSEAU_TEST_OPTIONS="--gnupg-home=$TROUSSEAU_TEST_GNUPG_HOME"
TROUSSEAU_COMMAND="$TROUSSEAU_BINARY_DIR/trousseau"

# Trousseau global context
TROUSSEAU_TEST_STORE="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}store"
TROUSSEAU_TEST_STORE_AES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}aes_store"
TROUSSEAU_TEST_STORE_CREATE="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}create_store"
TROUSSEAU_TEST_STORE_CREATE_AES="${TMP_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}aes_create_store"

# Include all the helpers
. "$DIR/keyring_helpers.bash"
. "$DIR/system_helpers.bash"
. "$DIR/gpg_helpers.bash"
. "$DIR/env_helpers.bash"


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
