package trousseau

import (
	"encoding/json"
	"errors"
	"fmt"
	crypto "github.com/oleiade/trousseau/crypto"
	openpgp "github.com/oleiade/trousseau/crypto/openpgp"
	"os"
	"reflect"
)

type Store struct {
	*DataStore
	path           string
	encryptionOpts *crypto.Options
}

type DataStore struct {
	KVStore
	Meta      Meta                   `json:"_meta"`
	Container map[string]interface{} `json:"data"`
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

type Encodable interface {
	FromJson([]byte) error
	ToJson() ([]byte, error)
}

// LoadStore reads and decrypt a trousseau data store from the
// provided path, using the provided encryption options.
func LoadStore(path string, opts *crypto.Options) (*Store, error) {
	var store *Store = NewStore(path, opts)
	var err error = nil

	switch opts.Algorithm {
	case crypto.GPG_ENCRYPTION:
		var f *openpgp.GpgFile
		var jsonData []byte

		f, err = openpgp.OpenFile(path, os.O_RDONLY, opts.Passphrase, opts.Recipients)
		if err != nil {
			return nil, fmt.Errorf("trousseau data store not found (%s)", err.(*os.PathError).Path)
		}
		defer f.Close()

		jsonData, err = f.ReadAll()
		if err != nil {
			return nil, err
		}

		err = store.FromJson(jsonData)
		if err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("Invalid encryption method provided")
	}

	return store, nil
}

// Sync encrypts the store content and writes it to the disk
func (s *Store) Sync() error {
	switch s.encryptionOpts.Algorithm {
	case crypto.GPG_ENCRYPTION:
		var f *openpgp.GpgFile
		var err error

		f, err = openpgp.OpenFile(s.path,
			os.O_CREATE|os.O_WRONLY,
			s.encryptionOpts.Passphrase,
			s.Meta.Recipients)
		if err != nil {
			return err
		}
		defer f.Close()

		jsonData, err := s.ToJson()
		if err != nil {
			return err
		}

		_, err = f.Write([]byte(jsonData))
		if err != nil {
			return err
		}
	default:
		return fmt.Errorf("Invalid encryption method provided")
	}

	return nil
}

func NewStore(path string, encryptionOpts *crypto.Options) *Store {
	return &Store{
		DataStore:      NewDataStore(),
		path:           path,
		encryptionOpts: encryptionOpts,
	}
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
func CreateStoreFile(path string, opts *crypto.Options, meta *Meta) (err error) {
	// if the store file already exists, return an error
	if _, err = os.Stat(path); err == nil {
		return errors.New("Store file already exists")
	}

	store := NewStore(path, opts)
	store.Meta = *meta

	err = store.Sync()
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

func (s *Store) Metadata() ([]KVPair, error) {
	metaType := reflect.TypeOf(s.DataStore.Meta)
	metaValue := reflect.ValueOf(s.DataStore.Meta)
	pairs := make([]KVPair, metaType.NumField())

	for i := 0; i < metaType.NumField(); i++ {
		key := metaType.Field(i).Name
		value := metaValue.FieldByName(key).Interface()
		pairs[i] = *NewKVPair(key, value)
	}

	return pairs, nil
}
