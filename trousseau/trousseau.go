package trousseau

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
)

type Trousseau struct {
	// Crypto public configuration attributes
	CryptoType      CryptoType      `json:"crypto_type"`
	CryptoAlgorithm CryptoAlgorithm `json:"crypto_algorithm"`

	// Encrypted data private attribute
	Data []byte `json:"_data"`

	// Crypto algorithm to decryption
	cryptoMapping map[CryptoAlgorithm]interface{}
}

func OpenTrousseau(fp string) (*Trousseau, error) {
	var trousseau *Trousseau
	var content []byte
	var err error

	content, err = ioutil.ReadFile(fp)
	if err != nil {
		return nil, err
	}

	err = json.Unmarshal(content, &trousseau)
	if err != nil {
		return nil, err
	}

	return trousseau, nil
}

func (t *Trousseau) Decrypt() (*Store, error) {
	var store *Store = &Store{
		Data: NewKVStore(),
	}

	switch t.CryptoAlgorithm {
	case GPG_ENCRYPTION:
		plainData, err := DecryptAsymmetricPGP(t.Data, GetPassphrase())
		if err != nil {
			return nil, err
		}

		err = json.Unmarshal(plainData, store)
		if err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("Invalid encryption method provided")
	}

	return store, nil
}

func (t *Trousseau) Encrypt(store *Store) error {
	switch t.CryptoAlgorithm {
	case GPG_ENCRYPTION:
		plainData, err := json.Marshal(*store)
		if err != nil {
			return err
		}

		t.Data, err = EncryptAsymmetricPGP(plainData, store.Meta.Recipients)
		if err != nil {
			return err
		}
	default:
		return fmt.Errorf("Invalid encryption method provided")
	}

	return nil
}

func (t *Trousseau) Write(fp string) error {
	jsonData, err := json.Marshal(t)
	if err != nil {
		return err
	}

	err = ioutil.WriteFile(fp, jsonData, os.FileMode(0700))
	if err != nil {
		return err
	}

	return nil
}
