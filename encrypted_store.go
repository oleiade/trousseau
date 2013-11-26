package trousseau

import (
	"errors"
	"fmt"
	"io/ioutil"
	"reflect"
)

type EncryptedStore struct {
	DataStore
	Data       string
	Encrypted  bool
	Passphrase string
}

func NewEncryptedStore(store *DataStore) *EncryptedStore {
	data, _ := store.ToJson()

	return &EncryptedStore{
		DataStore: *store,
		Data:      data,
		Encrypted: false,
	}
}

func NewEncryptedStoreFromFile(filePath, passphrase string) (*EncryptedStore, error) {
	encryptedData, err := ioutil.ReadFile(filePath)
	if err != nil {
		return nil, errors.New("trousseau data store file ($HOME/.trousseau) not found")
	}

	encryptedStore := &EncryptedStore{
		DataStore:  *NewDataStore(),
		Data:       string(encryptedData),
		Encrypted:  true,
		Passphrase: passphrase,
	}

	return encryptedStore, nil
}

func (es *EncryptedStore) Encrypt() (err error) {
	if es.Encrypted == true {
		return errors.New("EncryptedStore already encrypted")
	} else {
		jsonDataStore, err := es.DataStore.ToJson()
		if err != nil {
			return err
		}

		initPgp(gPubringFile, es.DataStore.Meta.Recipients)
		es.Data = encrypt(jsonDataStore)
		es.Encrypted = true
	}

	return nil
}

func (es *EncryptedStore) Decrypt() error {
	if es.Encrypted == false {
		return errors.New("EncryptedStore already decrypted")
	} else {
		var err error

		// Decrypt store data
		initCrypto(gSecringFile, es.Passphrase)
		es.Data, err = decrypt(es.Data, es.Passphrase)
		if err != nil {
			return err
		}

		// If decryption was succesful, set the encrypted
		// flag to false, and deserialize store content
		es.Encrypted = false
		err = es.DataStore.FromJson([]byte(es.Data))
		if err != nil {
			return err
		}
	}

	return nil
}

func (es *EncryptedStore) WriteToFile(filePath string) error {
	err := ioutil.WriteFile(filePath, []byte(es.Data), 0764)
	if err != nil {
		return err
	}

	return nil
}

func (es *EncryptedStore) Get(key string) (interface{}, error) {
	err := es.Decrypt()
	if err != nil {
		return nil, err
	}

	value, err := es.DataStore.Get(key)
	if err != nil {
		return nil, errors.New(fmt.Sprintf("Key %s does not exist", key))
	}
	es.DataStore.Meta.updateLastModificationMarker()

	err = es.Encrypt()
	if err != nil {
		return nil, err
	}

	return value, nil
}

func (es *EncryptedStore) Set(key string, value interface{}) error {
	err := es.Decrypt()
	if err != nil {
		return err
	}

	err = es.DataStore.Set(key, value)
	if err != nil {
		return err
	}
	es.DataStore.Meta.updateLastModificationMarker()

	err = es.Encrypt()
	if err != nil {
		return err
	}

	return nil
}

func (es *EncryptedStore) Del(key string) error {
	err := es.Decrypt()
	if err != nil {
		return err
	}

	es.DataStore.Del(key)
	es.DataStore.Meta.updateLastModificationMarker()

	err = es.Encrypt()
	if err != nil {
		return err
	}

	return nil
}

func (es *EncryptedStore) Keys() ([]string, error) {
	err := es.Decrypt()
	if err != nil {
		return nil, err
	}

	keys, _ := es.DataStore.Keys()

	err = es.Encrypt()
	if err != nil {
		return nil, err
	}

	return keys, nil
}

func (es *EncryptedStore) Items() ([]KVPair, error) {
	err := es.Decrypt()
	if err != nil {
		return nil, err
	}

	pairs, _ := es.DataStore.Items()

	err = es.Encrypt()
	if err != nil {
		return nil, err
	}

	return pairs, nil
}

func (es *EncryptedStore) Meta() ([]KVPair, error) {
	err := es.Decrypt()
	if err != nil {
		return nil, err
	}

	metaType := reflect.TypeOf(es.DataStore.Meta)
	metaValue := reflect.ValueOf(es.DataStore.Meta)
	pairs := make([]KVPair, metaType.NumField())

	for i := 0; i < metaType.NumField(); i++ {
		key := metaType.Field(i).Name
		value := metaValue.FieldByName(key).Interface()
		pairs[i] = *NewKVPair(key, value)
	}

	err = es.Encrypt()
	if err != nil {
		return nil, err
	}

	return pairs, nil
}
