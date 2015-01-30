#!/usr/bin/env bats


load helpers


@test "create store with one valid recipient succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE_CREATE create $TROUSSEAU_TEST_FIRST_KEY_ID
    [ "$status" -eq 0 ]
    [ -f $TROUSSEAU_TEST_STORE_CREATE ]
}

@test "create symmetric store succeeds" {
    run $TROUSSEAU_COMMAND --store $TROUSSEAU_TEST_STORE_CREATE_AES create --encryption-type 'symmetric'
    [ "$status" -eq 0 ]
    [ -f $TROUSSEAU_TEST_STORE_CREATE_AES ]
}

@test "create generates a file in 0600 mode" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE_CREATE create $TROUSSEAU_TEST_FIRST_KEY_ID
    [ "$status" -eq 0 ]
    [ -f $TROUSSEAU_TEST_STORE_CREATE ]

    # Now let's make sure the created file has proper mode (in a generic way)
    if [[ $(uname) == 'Linux' ]]; then
        run stat -c "%a" $TROUSSEAU_TEST_STORE_CREATE
        echo $output
        [ "$output" == "600" ]
    elif [[ $(uname) == 'Darwin' ]]; then
        run stat -f "%Mp%Lp" $TROUSSEAU_TEST_STORE_CREATE
        [ "$output" == "0600" ]
    fi
}

@test "create store with one invalid recipient fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE_CREATE create ABC123EAS

    [ "$status" -eq 1 ]
    [ ! -f $TROUSSEAU_TEST_STORE_CREATE ]
}

@test "create store without a recipient fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE_CREATE create 
    [ "$status" -eq 1 ]
}

@test "create store with one valid recipient and one invalid recipient fails" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE_CREATE create $TROUSSEAU_TEST_FIRST_KEY_ID ABC123EAS 

    [ "$status" -eq 1 ]
    [ ! -f $TROUSSEAU_TEST_STORE_CREATE ]
}

