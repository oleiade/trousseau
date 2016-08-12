#!/usr/bin/env bats

load test_helpers

@test "list-recipients succeeds" {
    run $TROUSSEAU_BIN --gnupg-home $TEMP_GNUPG_HOME --store $TEMP_GPG_STORE list-recipients
    
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "$TEMP_GNUPG_KEY_A_KEY_ID" ]
}
