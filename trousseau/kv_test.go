package trousseau

import (
	"testing"
)

func TestKVStoreGet(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123

	v, err := kvStore.Get("abc")
	ok(t, err)
	equals(t, v, 123)
}

func TestKVStoreGet_errors_on_non_existing_key(t *testing.T) {
	kvStore := NewKVStore()

	v, err := kvStore.Get("easy as")
	notOk(t, err)
	equals(t, v, "")
}

func TestKVStoreSet(t *testing.T) {
	kvStore := NewKVStore()

	kvStore.Set("abc", 123)
	equals(t, kvStore.Container["abc"], 123)
}

func TestKVStoreDel(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Set("abc", 123)

	kvStore.Del("abc")
	_, in := kvStore.Container["abc"]
	assert(t, in == false, "Expected 'abc' key to be removed from KVStore Container")
}

func TestKVStoreRename_without_overwrite_and_non_existing_destination_key(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123

	err := kvStore.Rename("abc", "easy as", false)
	_, srcIn := kvStore.Container["abc"]
	_, destIn := kvStore.Container["easy as"]

	ok(t, err)
	assert(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore.Container["easy as"], 123)
}

func TestKVStoreRename_without_overwrite_and_existing_destination_key(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123
	kvStore.Container["easy as"] = "do re mi"

	err := kvStore.Rename("abc", "easy as", false)
	_, srcIn := kvStore.Container["abc"]
	_, destIn := kvStore.Container["easy as"]

	notOk(t, err)
	assert(t, srcIn == true, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore.Container["easy as"], "do re mi")
}

func TestKVStoreRename_with_overwrite_and_existing_destination_key(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123
	kvStore.Container["easy as"] = "do re mi"

	err := kvStore.Rename("abc", "easy as", true)
	_, srcIn := kvStore.Container["abc"]
	_, destIn := kvStore.Container["easy as"]

	ok(t, err)
	assert(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore.Container["easy as"], 123)
}

func TestKVStoreRename_with_overwrite_and_non_existing_destination_key(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123

	err := kvStore.Rename("abc", "easy as", true)
	_, srcIn := kvStore.Container["abc"]
	_, destIn := kvStore.Container["easy as"]

	ok(t, err)
	assert(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert(t, destIn == true, "Expected destination key to be present in KVStore")
	equals(t, kvStore.Container["easy as"], 123)
}

func TestKVStoreKeys(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123
	kvStore.Container["easy as"] = "do re mi"

	keys := kvStore.Keys()
	equals(t, keys, []string{"abc", "easy as"})
}

func TestKVStoreItems(t *testing.T) {
	kvStore := NewKVStore()
	kvStore.Container["abc"] = 123
	kvStore.Container["easy as"] = "do re mi"

	items := kvStore.Items()
	expected := map[string]interface{}{
		"abc": 123,
		"easy as": "do re mi",
	}
	equals(t, items, expected)
}
