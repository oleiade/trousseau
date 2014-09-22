#!/usr/bin/env bats


load system_helpers
load keyring_helpers
load test_helpers


@test "create store with one valid recipient" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE_CREATE create $TROUSSEAU_TEST_KEY_ID

    [ "$status" -eq 0 ]
    [ -f $TROUSSEAU_TEST_STORE_CREATE ]
}

@test "create store with one invalid recipient" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE_CREATE create ABC123EAS

    [ "$status" -eq 1 ]
    [ ! -f $TROUSSEAU_TEST_STORE_CREATE ]
}

@test "create store without a recipient fails" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE_CREATE create 
    [ "$status" -eq 1 ]
}

@test "create store with one valid recipient and one invalid recipient" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE_CREATE create $TROUSSEAU_TEST_KEY_ID ABC123EAS 

    [ "$status" -eq 1 ]
    [ ! -f $TROUSSEAU_TEST_STORE_CREATE ]
}

