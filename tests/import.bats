#!/usr/bin/env bats

load helpers


# This file will be automatically collected at teardown
TEST_FILE="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}_outfile"


@test "import store from valid exported data store succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set abc 123
    cp $TROUSSEAU_TEST_STORE $TEST_FILE
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE del abc

    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE import $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]

    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE get abc
    [ "$status" -eq 0 ]
    [ "$output" == "123" ]
}

@test "import store from valid data on stdin succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set abc 123
    cp $TROUSSEAU_TEST_STORE $TEST_FILE
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE del abc

    cat $TEST_FILE | $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE import 
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE show
    [ "$status" -eq 0 ]
    [ "$output" == "abc : 123" ]
}

@test "import store from invalid file fails" {
    echo "invalid" "${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}_import_test"
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE import $TEST_FILE
    [ "$status" -eq 1 ]
}

@test "import non existing store file fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE import /non/existing/file
    [ "$status" -eq 1 ]
}
