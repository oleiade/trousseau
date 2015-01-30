#!/usr/bin/env bash

# Which binary to run the tests against. As a default
# this variable is set to the executable on your path.
# Export the $TROUSSEAU_BIN env variable to override it.
TROUSSEAU_BIN="${TROUSSEAU_BIN:=$(which trousseau)}"


# If any of the tests function needs to write some file,
# they should do it inside the destination pointed by
# $TROUSSEAU_TESTS_DIR. As a default this variable is
# set to /tmp/trousseau_tests.
# Export the $TROUSSEAU_TESTS_DIR env variable to override it.
TROUSSEAU_TESTS_DIR="${TROUSSEAU_TESTS_DIR:=/tmp/trousseau_tests}"



TEMP_AES_STORE="$TROUSSEAU_TESTS_DIR/aes_store"
TEMP_GPG_STORE="$TROUSSEAU_TESTS_DIR/gpg_store"

TEMP_ENCRYPTION_PASSPHRASE="trousseau"

setup() {
	mkdir $TROUSSEAU_TESTS_DIR

	export TROUSSEAU_PASSPHRASE=$TEMP_ENCRYPTION_PASSPHRASE

	# Create temporary trousseau data stores for test purpose
	$TROUSSEAU_BIN --store $TEMP_AES_STORE create --encryption-type 'symmetric'
}

teardown() {
    rm -rf $TROUSSEAU_TESTS_DIR
    unset $TROUSSEAU_PASSPHRASE
}