#!/usr/bin/env bats

load test_helpers


# This file will be automatically collected at teardown
TEST_FILE="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}_outfile"


@test "export store to valid destination succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]
}

@test "export without arguments prints to stdout" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export 
    [ "$status" -eq 0 ] 
    [ "$output" != "" ]
}

@test "export store creates a valid data store file" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export $TEST_FILE
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TEST_FILE keys

    [ "$status" -eq 0 ]
}

@test "export store to valid destination creates a file in 0600 mode" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export $TEST_FILE

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
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export /tmp

    [ "$status" -eq 1 ]
    [ ! -f $TEST_FILE ]
}

@test "export store to non existing destination fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export /does/not/exist

    [ "$status" -eq 1 ]
    [ ! -f $TEST_FILE ]
}

@test "export store to non sufficient rights destination fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE export /root

    [ "$status" -eq 1 ]
    [ ! -f $TEST_FILE ]
}
