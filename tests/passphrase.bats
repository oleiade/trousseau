#!/usr/bin/env bats

load test_helpers

@test "Opening store with passphrase set in environment succeeds" {
    # Remove it if it exists
    unset TROUSSEAU_PASSPHRASE

    export TROUSSEAU_PASSPHRASE=$TEMP_ENCRYPTION_PASSPHRASE
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE keys
    [ "$status" -eq 0 ]

    unset TROUSSEAU_PASSPHRASE
}

@test "Opening store without passphrase set in environment fails" {
    # Remove it if it exists
    unset TROUSSEAU_PASSPHRASE

    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE keys
    [ "$status" -eq 1 ]
}
