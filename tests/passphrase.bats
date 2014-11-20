#!/usr/bin/env bats

load helpers

@test "Opening store with passphrase set in environment succeeds" {
    # Remove it if it exists
    unset TROUSSEAU_PASSPHRASE

    export TROUSSEAU_PASSPHRASE=$TROUSSEAU_TEST_KEY_PASSPHRASE
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 0 ]

    unset TROUSSEAU_PASSPHRASE
}

@test "Opening store without passphrase set in environment fails" {
    # Remove it if it exists
    unset TROUSSEAU_PASSPHRASE

    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE keys
    [ "$status" -eq 1 ]
}
