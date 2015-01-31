#!/usr/bin/env bats

load test_helpers


@test "show values of an empty store succeeds" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE show
    [ "$status" -eq 0 ]
}

@test "show values of a fulfilled store succeeds" {
    # Prepare the data store and environement
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE show 
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "abc : 123" ]
}

