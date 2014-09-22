#!/usr/bin/env/bats

load system_helpers
load keyring_helpers
load test_helpers


@test "open data store with proper keyring service being set on osx succeeds" {
    # Make sure we remove the environment entry
    # for the test key passphrase
    unset TROUSSEAU_PASSPHRASE

    # Create proper keyring entry
    create_keyring_service
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 0 ]

    # Drop the keyring entry
    drop_keyring_service
}

@test "open data store with no keyring service set in environment on osx fails" {
    # Make sure we remove the environment entry
    # for the test key passphrase
    unset TROUSSEAU_PASSPHRASE
    
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 1 ]
}

@test "open data store with non existing keyring service set in environment on osx fails" {
    # Make sure we remove the environment entry
    # for the test key passphrase
    unset TROUSSEAU_PASSPHRASE

    export TROUSSEAU_KEYRING_SERVICE=nonexisting

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 1 ]
}

