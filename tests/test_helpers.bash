#!/usr/bin/env bash

# Working directory of the script
DIR="${BASH_SOURCE%/*}"
if [[ ! -d "$DIR" ]]; then DIR="$PWD"; fi

# Testing context
TEST_DIR=$DIR/tmp
TROUSSEAU_BINARY_DIR="$DIR/../bin"
TROUSSEAU_COMMAND="$TROUSSEAU_BINARY_DIR/trousseau"

# Include all the helpers
HELPERS_DIR=$DIR/helpers
shopt -s extglob
for helper in $(ls $HELPERS_DIR/*.bash)
do
    . $helper
done

setup_tmp_dir() {
    if [ ! -d $TEST_DIR ]
    then
        mkdir $TEST_DIR
    fi
}

teardown_tmp_dir() {
    if [ -d $TEST_DIR ]
    then
        rm -rf $TEST_DIR
    fi
}

# Setup and teardown
setup() {
    # Make sure to fail fast if trousseau was not built
    # and no binary path could be found
    if [ ! -d $TROUSSEAU_BINARY_DIR ] || [ ! -f $TROUSSEAU_COMMAND ]; then
        echo "whether trousseau binary dir ($TROUSSEAU_BINARY_DIR) or executable ($TROUSSEAU_COMMAND) not found" 
        exit 1
    fi

    setup_tmp_dir

    setup_ggp
    setup_env
    setup_store 'asymmetric'
    setup_store 'symmetric'
}

teardown() {
    teardown_gpg
    teardown_env
    teardown_store

    teardown_tmp_dir
}
