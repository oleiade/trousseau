#!/usr/bin/env bats

load test_helpers


# This file will be automatically collected at teardown
TEST_FILE="/tmp/${TROUSSEAU_TEST_FILES_PREFIX}_outfile"


@test "set valid key pair succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    [ "$status" -eq 0 ]
    
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get abc
    [ "${lines[0]}" = "123" ]
}

@test "set value pair with no value fails" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'foo' 

    [ "$status" -eq 1 ]
}

@test "set valid key's value import from file succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    echo "do re mi" >> $TEST_FILE

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'easy as' -f $TEST_FILE
    [ "$status" -eq 0 ]

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get 'easy as'
    [ "${lines[0]}" = "do re mi" ]
}

@test "set valid key's value import from non openable file fails" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'easy as' -f /tmp/non_existing_file

    [ "$status" -eq 1 ]
}


