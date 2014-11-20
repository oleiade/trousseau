#!/usr/bin/env bats

load helpers

@test "list-recipients succeeds" {
    run $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME --store $TROUSSEAU_TEST_STORE list-recipients
    
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "6F7FEB2D" ]
}
