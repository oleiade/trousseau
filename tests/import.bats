#!/usr/bin/env bats

load test_helpers


@test "import store from valid exported data store succeeds" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/import_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    cp $TEMP_GPG_STORE $TEST_FILE
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE del abc

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE import $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get abc
    [ "$status" -eq 0 ]
    [ "$output" == "123" ]
}

@test "import store from valid data on stdin succeeds" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/import_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    cp $TEMP_GPG_STORE $TEST_FILE
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE del abc

    cat $TEST_FILE | $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE import 
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE show
    [ "$status" -eq 0 ]
    [ "$output" == "abc : 123" ]
}

@test "import store from invalid file fails" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/import_out"

    echo "invalid" "${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}_import_test"
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE import $TEST_FILE
    [ "$status" -eq 1 ]
}

@test "import non existing store file fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE import /non/existing/file
    [ "$status" -eq 1 ]
}
