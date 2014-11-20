#!/usr/bin/env/bats

load helpers


@test "open data store with proper keyring service being set on osx succeeds" {
    # Run this test only on osx
    if [[ $(uname) != "Darwin" ]]; then
       skip "test should only be ran on osx"
    fi

    # Make sure we remove the environment entry
    # for the test key passphrase
    unset TROUSSEAU_PASSPHRASE

    # Create proper keyring entry
    setup_keyring_entry
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME

    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 0 ]

    # Drop the keyring entry
    teardown_keyring_entry
}

@test "open data store with no keyring service set in environment on osx fails" {
    # Run this test only on osx
    if [[ $(uname) != 'Darwin' ]]; then
        skip "test should only be performed on osx"
    fi

    # Make sure we remove the environment entry
    # for the test key passphrase
    unset TROUSSEAU_PASSPHRASE
    
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 1 ]
}

@test "open data store with non existing keyring service set in environment on osx fails" {
    # Run this test only on osx
    if [[ $(uname) != 'Darwin' ]]; then
        skip "test should only be performed on osx"
    fi

    # Make sure we remove the environment entry
    # for the test key passphrase
    unset TROUSSEAU_PASSPHRASE

    export TROUSSEAU_KEYRING_SERVICE=nonexisting

    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 1 ]
}

