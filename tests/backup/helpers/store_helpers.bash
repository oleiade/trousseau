#!/usr/bin/env bash


# Trousseau global context
TROUSSEAU_TEST_STORE="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}store"
TROUSSEAU_TEST_STORE_AES="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}aes_store"
TROUSSEAU_TEST_STORE_CREATE="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}create_store"
TROUSSEAU_TEST_STORE_CREATE_AES="${TEST_DIR}/${TROUSSEAU_TEST_FILES_PREFIX}aes_create_store"

setup_store() {
	if [ $1 == 'asymmetric' ]
	then
	    # Otherwise, create the base test store
	    $TROUSSEAU_COMMAND --gnupg-home $TROUSSEAU_TEST_GNUPG_HOME \
	                       --store $TROUSSEAU_TEST_STORE \
	                       create $TROUSSEAU_TEST_FIRST_KEY_ID >&2 
	elif [ $1 == 'symmetric' ]
	then
    	$TROUSSEAU_COMMAND --store $TROUSSEAU_TEST_STORE_AES \
                       	   create --encryption-type 'symmetric' >&2 
    else
    	echo "No data store type supplied. Unable to create test trousseau store" >&2
    	exit 1
    fi
}

teardown_store() {
	if [ -f $TROUSSEAU_TEST_STORE ]; then
		rm $TROUSSEAU_TEST_STORE
	fi

	if [ -f $TROUSSEAU_TEST_STORE_AES ]; then
		rm $TROUSSEAU_TEST_STORE_AES
	fi
}