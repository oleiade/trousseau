package trousseau

import (
	"testing"
)

func TestKVStoreGet(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123

	v, err := kvStore.Get("abc")
	ok(t, err)
	equals(t, v, 123)
}

func TestKVStoreGet_errors_on_non_existing_key(t *testing.T) {
	kvStore := make(KVStore)

	v, err := kvStore.Get("easy as")
	notOk(t, err)
	equals(t, v, "")
}

func TestKVStoreSet(t *testing.T) {
	kvStore := make(KVStore)

	kvStore.Set("abc", 123)
	equals(t, kvStore["abc"], 123)
}

func TestKVStoreDel(t *testing.T) {
	kvStore := make(KVStore)
	kvStore.Set("abc", 123)

	kvStore.Del("abc")
	_, in := kvStore["abc"]
	assert(t, in == false, "Expected 'abc' key to be removed from KVStore Container")
}

func TestKVStoreRename_without_overwrite_and_non_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123

	err := kvStore.Rename("abc", "easy as", false)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	ok(t, err)
	assert(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore["easy as"], 123)
}

func TestKVStoreRename_without_overwrite_and_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	err := kvStore.Rename("abc", "easy as", false)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	notOk(t, err)
	assert(t, srcIn == true, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore["easy as"], "do re mi")
}

func TestKVStoreRename_with_overwrite_and_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	err := kvStore.Rename("abc", "easy as", true)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	ok(t, err)
	assert(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore["easy as"], 123)
}

func TestKVStoreRename_with_overwrite_and_non_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123

	err := kvStore.Rename("abc", "easy as", true)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	ok(t, err)
	assert(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore["easy as"], 123)
}

func TestKVStoreKeys(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	keys := kvStore.Keys()
	equals(t, keys, []string{"abc", "easy as"})
}

func TestKVStoreItems(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	items := kvStore.Items()
	expected := map[string]interface{}{
		"abc":     123,
		"easy as": "do re mi",
	}
	equals(t, items, expected)
}
