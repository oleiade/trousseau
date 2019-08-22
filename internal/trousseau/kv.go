package trousseau

import (
	"errors"
	"fmt"
	"sort"
)

type KVStore map[string]interface{}

// Get method fetches a key from the trousseau file store
func (kvs *KVStore) Get(key string) (interface{}, error) {
	value, ok := (*kvs)[key]
	if !ok {
		return "", errors.New(fmt.Sprintf("Key %s does not exist", key))
	}

	return value, nil
}

// Set method sets a key value pair in the store
func (kvs *KVStore) Set(key string, value interface{}) {
	(*kvs)[key] = value
}

// Del deletes a key value pair from the trousseau file
func (kvs *KVStore) Del(key string) {
	delete((*kvs), key)
}

// Rename a data store key to dest. If overwrite parameter
// is provided with a true value, any existing destination key-value
// pair will be overriden, otherwise (false) an error will be returned.
func (kvs *KVStore) Rename(src, dest string, overwrite bool) error {

	srcValue, ok := (*kvs)[src]
	if !ok {
		return errors.New(fmt.Sprintf("Source key %s does not exist", src))
	}

	// If destination key already exists, and overwrite flag is
	// set to false, then return an error
	_, ok = (*kvs)[dest]
	if ok && overwrite == false {
		return errors.New(fmt.Sprintf("Destination key %s already exists an overwrite flag was not provided.", dest))
	}

	// Otherwise update dest value
	kvs.Set(dest, srcValue)
	kvs.Del(src)

	return nil
}

// Keys lists the keys contained in the trousseau store file
func (kvs *KVStore) Keys() []string {
	index := 0
	keys := make([]string, len((*kvs)))

	for key, _ := range *kvs {
		keys[index] = key
		index++
	}

	// Sort in alphabetical order
	sort.Strings(keys)
	return keys
}

func (kvs *KVStore) Items() map[string]interface{} {
	items := make(map[string]interface{})

	for key, value := range *kvs {
		items[key] = value
	}

	return items
}
