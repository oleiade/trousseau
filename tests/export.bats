#!/usr/bin/env bats

load test_helpers


@test "export store to valid destination succeeds" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/export_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]
}

@test "export without arguments prints to stdout" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export 
    [ "$status" -eq 0 ] 
    [ "$output" != "" ]
}

@test "export store creates a valid data store file" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/export_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export $TEST_FILE
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEST_FILE keys

    [ "$status" -eq 0 ]
}

@test "export store to valid destination creates a file in 0600 mode" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/export_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export $TEST_FILE

    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]

    # Now let's make sure the created file has proper mode (in a generic way)
    if [[ $(uname) == 'Linux' ]]; then
        run stat -c "%a" $TEST_FILE
        [ "$output" == "600" ]
    elif [[ $(uname) == 'Darwin' ]]; then
        run stat -f "%Mp%Lp" $TEST_FILE
        [ "$output" == "0600" ]
    fi
}

@test "export store to directory destination fails" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/export_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export /tmp

    [ "$status" -eq 1 ]
    [ ! -f $TEST_FILE ]
}

@test "export store to non existing destination fails" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/export_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export /does/not/exist

    [ "$status" -eq 1 ]
    [ ! -f $TEST_FILE ]
}

@test "export store to non sufficient rights destination fails" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/export_out"

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE export /root

    [ "$status" -eq 1 ]
    [ ! -f $TEST_FILE ]
}
