#!/usr/bin/env bash


# Path of the current helpers file. Used essentialy
# to produce tests dir relative paths such as the test
# gnupg keys dir.
DIR="${BASH_SOURCE%/*}"
if [[ ! -d "$DIR" ]]; then DIR="$PWD"; fi


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


# Define where to create test trousseau data stores 
TEMP_AES_STORE="$TROUSSEAU_TESTS_DIR/aes_store"
TEMP_GPG_STORE="$TROUSSEAU_TESTS_DIR/gpg_store"


# Directory where are located the test gnupg keys.
# Should be relative to helpers file location, AKA $DIR.
TEMP_GNUPG_TESTS_KEYS_DIR="$DIR/keys"


# Temporary directory where we're gonna store
# the test gnupg data, and install our test keys
TEMP_GNUPG_HOME="$TROUSSEAU_TESTS_DIR/gnupg"


# Gnupg key A attributes definition
TEMP_GNUPG_KEY_A_PUB_FILE="${TEMP_GNUPG_TESTS_KEYS_DIR}/trousseau_public_first_test.key"
TEMP_GNUPG_KEY_A_SEC_FILE="${TEMP_GNUPG_TESTS_KEYS_DIR}/trousseau_private_first_test.key"
TEMP_GNUPG_KEY_A_KEY_ID="6F7FEB2D"
TEMP_GNUPG_KEY_A_KEY_EMAIL="theo@trousseau.io"


# Gnupg key B attributes definition
TEMP_GNUPG_KEY_B_PUB_FILE="${TEMP_GNUPG_TESTS_KEYS_DIR}/trousseau_public_second_test.key"
TEMP_GNUPG_KEY_B_SEC_FILE="${TEMP_GNUPG_TESTS_KEYS_DIR}/trousseau_private_second_test.key"
TEMP_GNUPG_KEY_B_KEY_ID="EA7F9C59"
TEMP_GNUPG_KEY_B_KEY_EMAIL="theo@trousseau.io"


# Whether created asymmetricaly or symmetricaly, always use the same
# data store/gpg key passphrase
TEMP_ENCRYPTION_PASSPHRASE="trousseau"


# Defines the test keyring entry name to be used
TEMP_KEYRING_ENTRY_NAME="trousseau_test"


# polite_sudo exposes a verbose and explicit sudo command
# so any test who might need to ask for sudo password
# would be more user-friendly.
polite_sudo() {
    sudo -p "Bats testing framework requires sudo access to setup the test key passphrase in keychain. Password: " "$@"
}


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
        polite_sudo security add-generic-password -a "${USER}" -s "${TEMP_KEYRING_ENTRY_NAME}" -w "${TEMP_ENCRYPTION_PASSPHRASE}" >&2
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
        polite_sudo security delete-generic-password -a "${USER}" -s "${TEMP_KEYRING_ENTRY_NAME}" >&2
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}


setup() {
	mkdir $TROUSSEAU_TESTS_DIR

	export TROUSSEAU_PASSPHRASE=$TEMP_ENCRYPTION_PASSPHRASE

    # Create the temporary gnupg keyring and import test keys into it
    mkdir -m 0700 $TEMP_GNUPG_HOME
    gpg --quiet --homedir $TEMP_GNUPG_HOME --import $TEMP_GNUPG_KEY_A_PUB_FILE >&2
    gpg --quiet --homedir $TEMP_GNUPG_HOME --allow-secret-key-import --import $TEMP_GNUPG_KEY_A_SEC_FILE >&2 
    gpg --quiet --homedir $TEMP_GNUPG_HOME --import $TEMP_GNUPG_KEY_B_PUB_FILE >&2 
    gpg --quiet --homedir $TEMP_GNUPG_HOME --allow-secret-key-import --import $TEMP_GNUPG_KEY_B_SEC_FILE >&2 

	# Create temporary trousseau data stores for test purpose
	$TROUSSEAU_BIN --store $TEMP_AES_STORE create --encryption-type 'symmetric' > /dev/null
	$TROUSSEAU_BIN --store $TEMP_GPG_STORE --gnupg-home $TEMP_GNUPG_HOME create $TEMP_GNUPG_KEY_A_KEY_ID > /dev/null
}


teardown() {
    rm -rf $TROUSSEAU_TESTS_DIR
    unset $TROUSSEAU_PASSPHRASE
}