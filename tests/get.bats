#!/usr/bin/env bats

load test_helpers

# This file will be automatically collected at teardown
TEST_FILE="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}_outfile"


@test "get valid key pair succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set abc 123
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE get abc
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "123" ]
}

@test "get valid key pair succeeds with symmetric encryption" {
    run $TROUSSEAU_COMMAND --store $TROUSSEAU_TEST_STORE_AES set abc 123
    run $TROUSSEAU_COMMAND --store $TROUSSEAU_TEST_STORE_AES get abc
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "123" ]
}

@test "get value pair of non existing key fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE get 'foo' 

    [ "$status" -eq 1 ]
}

@test "get value pair of non existing key fails with symmetric encryption" {
    run $TROUSSEAU_COMMAND --store $TROUSSEAU_TEST_STORE_AES get 'foo' 

    [ "$status" -eq 1 ]
}

@test "get valid key value's export to file succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE get 'easy as' -f $TEST_FILE
    [ "$status" -eq 0 ]
    [ -f $TEST_FILE ]

    run cat $TEST_FILE
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "do re mi" ]
}

@test "get valid key value's export to file with mode 0600" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE get 'easy as' -f $TEST_FILE
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

@test "get valid key value's export to non openable file fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set 'easy as' 'do re mi'
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE get 'easy as' -f /root

    [ "$status" -eq 1 ]
}

