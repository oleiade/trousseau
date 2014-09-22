#!/usr/bin/env bash


create_keyring_service() {
    platform=$(uname)

    if [[ $platform == 'Linux' ]]; then
        platform='linux'
    elif [[ $platform == 'Darwin' ]]; then
        polite_sudo security add-generic-password -a "${USER}" -s "${TROUSSEAU_KEYRING_SERVICE_NAME}" -w "${TROUSSEAU_TEST_KEY_PASSPHRASE}" &> /dev/null
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}

drop_keyring_service() {
    platform=$(uname)

    if [[ $platform == 'Linux' ]]; then
        platform='linux'
    elif [[ $platform == 'Darwin' ]]; then
        polite_sudo security delete-generic-password -a "${USER}" -s "${TROUSSEAU_KEYRING_SERVICE_NAME}" &> /dev/null
    elif [[ $platform == 'FreeBSD' ]]; then
        platform='freebsd'
    fi
}
