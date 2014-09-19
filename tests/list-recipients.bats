#!/usr/bin/env bats

load test_helpers

@test "list-recipients succeeds" {
    export TROUSSEAU_KEYRING_SERVICE=$TROUSSEAU_KEYRING_SERVICE_NAME
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE list-recipients
    
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "6F7FEB2D" ]
}
