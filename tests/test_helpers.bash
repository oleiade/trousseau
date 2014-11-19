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
TROUSSEAU_COMMAND="$TROUSSEAU_BINARY_DIR/trousseau"

# Include all the helpers
. "$DIR/keyring_helpers.bash"
. "$DIR/system_helpers.bash"
. "$DIR/gpg_helpers.bash"
. "$DIR/env_helpers.bash"
. "$DIR/store_helpers.bash"

# Setup and teardown
setup() {
    # Make sure to fail fast if trousseau was not built
    # and no binary path could be found
    if [ ! -d $TROUSSEAU_BINARY_DIR ] || [ ! -f $TROUSSEAU_COMMAND ]; then
        echo "whether trousseau binary dir ($TROUSSEAU_BINARY_DIR) or executable ($TROUSSEAU_COMMAND) not found" 
        exit 1
    fi

    setup_ggp
    setup_env
    setup_store 'asymmetric'
    setup_store 'symmetric'
}

teardown() {
    teardown_gpg
    teardown_env
    teardown_store

    # Remove every trousseau test prefixed files from 
    # tmp dir
    rm -rf $TROUSSEAU_TEST_FILES
}
