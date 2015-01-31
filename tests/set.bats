#!/usr/bin/env bats

load test_helpers


@test "set valid key pair succeeds" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    [ "$status" -eq 0 ]
    
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get abc
    [ "${lines[0]}" = "123" ]
}

@test "set valid key pair succeeds with symmetric encryption" {
    run $TROUSSEAU_BIN --store $TEMP_AES_STORE set abc 123
    [ "$status" -eq 0 ]
    
    run $TROUSSEAU_BIN --store $TEMP_AES_STORE get abc
    [ "${lines[0]}" = "123" ]
}

#@test "set value pair with no value fails" {
#    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set 'foo' 

#    [ "$status" -eq 1 ]
#}

@test "set valid key's value import from file succeeds" {
    local TEST_FILE="$TROUSSEAU_TESTS_DIR/set_out"
    echo "do re mi" >> $TEST_FILE

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set 'easy as' -f $TEST_FILE
    [ "$status" -eq 0 ]

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get 'easy as'
    [ "${lines[0]}" = "do re mi" ]
}

@test "set valid key's value import from non openable file fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set 'easy as' -f ${TROUSSEAU_TESTS_DIR}/non_existing_file

    [ "$status" -eq 1 ]
}
