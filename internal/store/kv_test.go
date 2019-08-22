package store

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestKVStoreGet(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123

	v, err := kvStore.Get("abc")
	assert.Nil(t, err)
	assert.Equal(t, v, 123)
}

func TestKVStoreGet_errors_on_non_existing_key(t *testing.T) {
	kvStore := make(KVStore)

	v, err := kvStore.Get("easy as")
	assert.NotNil(t, err)
	assert.Equal(t, v, "")
}

func TestKVStoreSet(t *testing.T) {
	kvStore := make(KVStore)

	kvStore.Set("abc", 123)
	assert.Equal(t, kvStore["abc"], 123)
}

func TestKVStoreDel(t *testing.T) {
	kvStore := make(KVStore)
	kvStore.Set("abc", 123)

	kvStore.Del("abc")
	_, in := kvStore["abc"]
	assert.True(t, in == false, "Expected 'abc' key to be removed from KVStore Container")
}

func TestKVStoreRename_without_overwrite_and_non_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123

	err := kvStore.Rename("abc", "easy as", false)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	assert.Nil(t, err)
	assert.True(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert.True(t, destIn == true, "Expected destination key to be present in KVStore")
	assert.Equal(t, kvStore["easy as"], 123)
}

func TestKVStoreRename_without_overwrite_and_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	err := kvStore.Rename("abc", "easy as", false)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	assert.NotNil(t, err)
	assert.True(t, srcIn == true, "Expected source key to have been removed from KVStore")
	assert.True(t, destIn == true, "Expected destination key to be present in KVStore")
	assert.Equal(t, kvStore["easy as"], "do re mi")
}

func TestKVStoreRename_with_overwrite_and_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	err := kvStore.Rename("abc", "easy as", true)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	assert.Nil(t, err)
	assert.True(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert.True(t, destIn == true, "Expected destination key to be present in KVStore")
	assert.Equal(t, kvStore["easy as"], 123)
}

func TestKVStoreRename_with_overwrite_and_non_existing_destination_key(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123

	err := kvStore.Rename("abc", "easy as", true)
	_, srcIn := kvStore["abc"]
	_, destIn := kvStore["easy as"]

	assert.Nil(t, err)
	assert.True(t, srcIn == false, "Expected source key to have been removed from KVStore")
	assert.True(t, destIn == true, "Expected destination key to be present in KVStore")
	assert.Equal(t, kvStore["easy as"], 123)
}

func TestKVStoreKeys(t *testing.T) {
	kvStore := make(KVStore)
	kvStore["abc"] = 123
	kvStore["easy as"] = "do re mi"

	keys := kvStore.Keys()
	assert.Equal(t, keys, []string{"abc", "easy as"})
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
	assert.Equal(t, items, expected)
}
