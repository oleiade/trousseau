#!/usr/bin/env bats

load helpers


@test "show values of an empty store succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE show
    [ "$status" -eq 0 ]
}

@test "show values of a fulfilled store succeeds" {
    # Prepare the data store and environement
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE set abc 123
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE show 
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "abc : 123" ]
}

