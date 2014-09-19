#!/usr/bin/env bats

load test_helpers

@test "list keys of an empty store succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys

    [ "$status" -eq 0 ]
}

@test "list keys of a fulfilled store succeeds" {
    # Prepare the data store and environement
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE keys

    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "abc" ]
}

