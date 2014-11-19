#!/usr/bin/env bash

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