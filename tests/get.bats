#!/usr/bin/env bats

load test_helpers

# This file will be automatically collected at teardown
TEST_FILE="/tmp/${TROUSSEAU_TEST_FILES_PREFIX}_outfile"


@test "get valid key pair succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get abc
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "123" ]
}

@test "get value pair of non existing key fails" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get 'foo' 

    [ "$status" -eq 1 ]
}

@test "get valid key value's export to file succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get 'easy as' -f $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]

    run cat $TEST_FILE
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "do re mi" ]
}

@test "get valid key value's export to non openable file fails" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get 'easy as' -f /root

    [ "$status" -eq 1 ]
}

