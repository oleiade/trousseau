#!/usr/bin/env/bats


load test_helpers


@test "open data store with proper keyring service being set on osx succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys

    [ "$status" -eq 0 ]
}

@test "open data store with no keyring service set in environment on osx fails" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys

    [ "$status" -eq 1 ]
}

@test "open data store with non existing keyring service set in environment on osx fails" {
    export TROUSSEAU_KEYRING_SERVICE=nonexisting
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys

    [ "$status" -eq 1 ]
}

