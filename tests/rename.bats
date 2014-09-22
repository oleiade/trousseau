#!/usr/bin/env/bats

load system_helpers
load keyring_helpers
load test_helpers

@test "rename existing source key to non existing destination key succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE rename abc 'easy as'
    [ "$status" -eq 0 ]

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get abc
    [ "$status" -eq 1 ]

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get 'easy as'
    [ "$status" -eq 0 ]
    [ "$output" = "123" ]
}

@test "rename non existing source key fails" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE rename abc 'easy as'
    [ "$status" -eq 1 ]

}

@test "rename existing source key to existing destination key with overwrite flag succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE rename --overwrite abc 'easy as'
    [ "$status" -eq 0 ]

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get abc
    [ "$status" -eq 1 ]

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get 'easy as'
    [ "$status" -eq 0 ]
    [ "$output" = "123" ]
}

@test "rename existing source key to existing destination key without overwrite flag fails" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE rename abc 'easy as'
    [ "$status" -eq 1 ]

}
