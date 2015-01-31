#!/usr/bin/env bats


load test_helpers


@test "create store with one valid recipient succeeds" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store "${TEMP_GPG_STORE}_create" create $TEMP_GNUPG_KEY_A_KEY_ID
    [ "$status" -eq 0 ]
    [ -f "${TEMP_GPG_STORE}_create" ]
}

@test "create symmetric store succeeds" {
    run $TROUSSEAU_BIN --store "${TEMP_AES_STORE}_create" create --encryption-type symmetric
    [ "$status" -eq 0 ]
    [ -f "${TEMP_AES_STORE}_create" ]
}

@test "create generates a file in 0600 mode" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store "${TEMP_GPG_STORE}_create" create $TEMP_GNUPG_KEY_A_KEY_ID
    [ "$status" -eq 0 ]
    [ -f "${TEMP_GPG_STORE}_create" ]

    # Now let's make sure the created file has proper mode (in a generic way)
    if [[ $(uname) == 'Linux' ]]; then
        run stat -c "%a" "${TEMP_GPG_STORE}_create"
        echo $output
        [ "$output" == "600" ]
    elif [[ $(uname) == 'Darwin' ]]; then
        run stat -f "%Mp%Lp" "${TEMP_GPG_STORE}_create"
        [ "$output" == "0600" ]
    fi
}

@test "create store with one invalid recipient fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store "${TEMP_GPG_STORE}_create" create ABC123EAS

    [ "$status" -eq 1 ]
    [ ! -f "${TEMP_GPG_STORE}_create" ]
}

@test "create store without a recipient fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store "${TEMP_GPG_STORE}_create" create 
    [ "$status" -eq 1 ]
}

@test "create store with one valid recipient and one invalid recipient fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store "${TEMP_GPG_STORE}_create" create $TEMP_GNUPG_KEY_A_KEY_ID ABC123EAS 

    [ "$status" -eq 1 ]
    [ ! -f "${TEMP_GPG_STORE}_create" ]
}

