#!/usr/bin/env/bats

load test_helpers

@test "rename existing source key to non existing destination key succeeds" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE rename abc 'easy as'
    [ "$status" -eq 0 ]

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get abc
    [ "$status" -eq 1 ]

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get 'easy as'
    [ "$status" -eq 0 ]
    [ "$output" = "123" ]
}

@test "rename non existing source key fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE rename abc 'easy as'
    [ "$status" -eq 1 ]

}

@test "rename existing source key to existing destination key with overwrite flag succeeds" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set 'easy as' 'do re mi'

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE rename --overwrite abc 'easy as'
    [ "$status" -eq 0 ]

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get abc
    [ "$status" -eq 1 ]

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE get 'easy as'
    [ "$status" -eq 0 ]
    [ "$output" = "123" ]
}

@test "rename existing source key to existing destination key without overwrite flag fails" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set abc 123
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE set 'easy as' 'do re mi'

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE rename abc 'easy as'
    [ "$status" -eq 1 ]
}
