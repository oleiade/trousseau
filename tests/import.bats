#!/usr/bin/env bats

load test_helpers


# This file will be automatically collected at teardown
TEST_FILE="/tmp/${TROUSSEAU_TEST_FILES_PREFIX}_outfile"


@test "import store from valid exported data store succeeds" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE set abc 123
    cp $TROUSSEAU_TEST_STORE $TEST_FILE
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE del abc

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE import $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]

    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE get abc
    [ "$status" -eq 0 ]
    [ "$output" == "123" ]
}

@test "import store from invalid file fails" {
    echo "invalid" "/tmp/${TROUSSEAU_TEST_FILES_PREFIX}_import_test"
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE import $TEST_FILE
    [ "$status" -eq 1 ]
}

@test "import non existing store file fails" {
    run $TROUSSEAU_BINARY --store $TROUSSEAU_TEST_STORE import /non/existing/file
    [ "$status" -eq 1 ]
}
