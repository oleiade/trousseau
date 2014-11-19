#!/usr/bin/env bash

# setup_env exports tests common environment variables
setup_env() {
    export TROUSSEAU_PASSPHRASE=$TROUSSEAU_TEST_KEY_PASSPHRASE
}

# teardown_env cleans the environment from tests variables
teardown_env() {
    unset TROUSSEAU_PASSPHRASE
}
