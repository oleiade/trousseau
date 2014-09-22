#!/usr/bin/env bats

load test_helpers


@test "show values of an empty store succeeds" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE show

    [ "$status" -eq 0 ]
}

@test "show values of a fulfilled store" {
    # Prepare the data store and environement
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE show 
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "abc : 123" ]
}

