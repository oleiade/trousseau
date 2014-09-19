#!/usr/bin/env bats

load test_helpers


@test "show values of an empty store succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE show

    [ "$status" -eq 0 ]
}

@test "show values of a fulfilled store" {
    # Prepare the data store and environement
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE show 
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "abc : 123" ]
}

