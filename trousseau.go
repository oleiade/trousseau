package trousseau

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"

	"github.com/oleiade/serrure/aes"
	"github.com/oleiade/serrure/openpgp"
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
		// Check if the content of the file matches with a legacy
		// data store file format. Raise a proper error accordingly.
		contentVersion := DiscoverVersion(content, VersionDiscoverClosures)
		if contentVersion != "" {
			return nil, fmt.Errorf("outdated data store file format detected: %s. "+
				"You are currently using incompatible version: %s. "+
				"Please upgrade the data store by using the upgrade command.",
				contentVersion, TROUSSEAU_VERSION)
		}
		return nil, err
	}

	return trousseau, nil
}

func FromBytes(d []byte) (*Trousseau, error) {
	var trousseau *Trousseau
	var err error

	err = json.Unmarshal(d, &trousseau)
	if err != nil {
		// Check if the content of the file matches with a legacy
		// data store file format. Raise a proper error accordingly.
		contentVersion := DiscoverVersion(d, VersionDiscoverClosures)
		if contentVersion != "" {
			return nil, fmt.Errorf("outdated data store file format detected: %s. "+
				"You are currently using incompatible version: %s. "+
				"Please upgrade the data store by using the upgrade command.",
				contentVersion, TROUSSEAU_VERSION)
		}
		return nil, err
	}

	return trousseau, err
}

func (t *Trousseau) Decrypt() (*Store, error) {
	var store Store

	switch t.CryptoAlgorithm {
	case GPG_ENCRYPTION:
		passphrase, err := GetPassphrase()
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		d, err := openpgp.NewOpenPGPDecrypter(GnupgSecring(), passphrase)
		if err != nil {
			return nil, err
		}

		pd, err := d.Decrypt(t.Data)
		if err != nil {
			return nil, err
		}

		err = json.Unmarshal(pd, &store)
		if err != nil {
			return nil, err
		}
	case AES_256_ENCRYPTION:
		passphrase, err := GetPassphrase()
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		d := aes.NewAES256Decrypter(passphrase)
		pd, err := d.Decrypt(t.Data)
		if err != nil {
			return nil, err
		}

		err = json.Unmarshal(pd, &store)
		if err != nil {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Invalid encryption method provided")
	}

	return &store, nil
}

func (t *Trousseau) Encrypt(store *Store) error {
	switch t.CryptoAlgorithm {
	case GPG_ENCRYPTION:
		pd, err := json.Marshal(*store)
		if err != nil {
			return err
		}

		e, err := openpgp.NewOpenPGPEncrypter(GnupgPubring(), store.Meta.Recipients)
		if err != nil {
			return err
		}

		t.Data, err = e.Encrypt(pd)
		if err != nil {
			return err
		}
	case AES_256_ENCRYPTION:
		pd, err := json.Marshal(*store)
		if err != nil {
			return err
		}

		passphrase, err := GetPassphrase()
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		d, err := aes.NewAES256Encrypter(passphrase, nil)
		if err != nil {
			return err
		}

		t.Data, err = d.Encrypt(pd)
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

	err = ioutil.WriteFile(fp, jsonData, os.FileMode(0600))
	if err != nil {
		return err
	}

	return nil
}
