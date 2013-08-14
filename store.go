package trousseau

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
)

type Encodable interface {
	FromJson([]byte) error
	ToJson() ([]byte, error)
}

type KVStore interface {
	Get(string) (interface{}, error)
	Set(string, interface{}) error
	Del(string) error
	Keys() ([]string, error)
	Items() ([]KVPair, error)
}

type KVPair struct {
	Key   string
	Value interface{}
}

type DataStore struct {
	Meta      Meta                   `json:"_meta"`
	Container map[string]interface{} `json:"data"`
}

func NewKVPair(key string, value interface{}) *KVPair {
	return &KVPair{
		Key:   key,
		Value: value,
	}
}

func NewDataStore() *DataStore {
	return &DataStore{
		Container: make(map[string]interface{}),
	}
}

func (ds *DataStore) ToJson() (string, error) {
	encodedStore, err := json.Marshal(ds)
	if err != nil {
		return "", err
	}

	return string(encodedStore), nil
}

func (ds *DataStore) FromJson(jsonData []byte) error {
	err := json.Unmarshal(jsonData, &ds)
	if err != nil {
		return err
	}

	return nil
}

// CreateStoreFile creates a trousseau file at $HOME/.trousseau
func CreateStoreFile(path string, meta *Meta) (err error) {
	// if the store file already exists, return an error
	if _, err = os.Stat(path); err == nil {
		return errors.New("Store file already exists")
	}

	store := NewDataStore()
	store.Meta = *meta

	encryptedStore := NewEncryptedStore(store)
	encryptedStore.Encrypt()
	err = encryptedStore.WriteToFile(path)
	if err != nil {
		return err
	}

	return nil
}

// GetStoreKey fetches a key from the trousseau file store
func (ds *DataStore) Get(key string) (interface{}, error) {
	value, ok := ds.Container[key]
	if !ok {
		return "", errors.New(fmt.Sprintf("Key %s does not exist", key))
	}

	return value, nil
}

// SetStoreKey sets a key value pair in the store
func (ds *DataStore) Set(key string, value interface{}) error {
	ds.Container[key] = value

	return nil
}

// DelStoreKey deletes a key value pair from the trousseau file
func (ds *DataStore) Del(key string) error {
	delete(ds.Container, key)

	return nil
}

// ListStoreKeys lists the keys contained in the trousseau store file
func (ds *DataStore) Keys() ([]string, error) {
	index := 0
	keys := make([]string, len(ds.Container))

	for key, _ := range ds.Container {
		keys[index] = key
		index++
	}

	return keys, nil
}

func (ds *DataStore) Items() ([]KVPair, error) {
	index := 0
	pairs := make([]KVPair, len(ds.Container))

	for key, value := range ds.Container {
		pairs[index] = *NewKVPair(key, value)
		index++
	}

	return pairs, nil
}
