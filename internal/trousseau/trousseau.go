package trousseau

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"

	"github.com/mcuadros/go-defaults"
	"github.com/oleiade/serrure/aes"
	"github.com/oleiade/serrure/openpgp"
	"github.com/oleiade/trousseau/internal/config"
	"github.com/oleiade/trousseau/internal/store"
)

const TROUSSEAU_VERSION = "0.4.1"

type Vault struct {
	// Crypto public configuration attributes
	CryptoType      CryptoType      `json:"crypto_type" default:"asymmetric"`
	CryptoAlgorithm CryptoAlgorithm `json:"crypto_algorithm" default:"gpg"`

	// Encrypted data private attribute
	Data []byte `json:"_data"`

	// Crypto algorithm to decryption
	cryptoMapping map[CryptoAlgorithm]interface{}
}

// NewVault initializes a vault with default values
func NewVault() *Vault {
	vault := new(Vault)
	defaults.SetDefaults(vault)
	return vault
}

// OpenVault loads a Vault from a from a file.
// It returns an error if the loaded Vault's version is not compatible
// with the version of trousseau currently being used
func OpenVault(fp string) (*Vault, error) {
	var content []byte
	content, err := ioutil.ReadFile(fp)
	if err != nil {
		return nil, err
	}

	var vault *Vault
	err = json.Unmarshal(content, &vault)
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

	return vault, nil
}

// FromBytes loads a Vault from a slice of bytes.
// It returns an error if the loaded Vault's version is not compatible
// with the version of trousseau currently being used
func FromBytes(d []byte) (*Vault, error) {
	var vault *Vault
	err := json.Unmarshal(d, &vault)
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

	return vault, err
}

// Decrypt deciphers the encrypted part of the Vault, and returns the
// unencrypted Store.
func (t *Vault) Decrypt(c *config.Config) (*store.Store, error) {
	var store store.Store

	switch t.CryptoAlgorithm {
	case GPG_ENCRYPTION:
		passphrase, err := GetPassphrase(c)
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
		passphrase, err := GetPassphrase(c)
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

// Encrypt encrypts a Store into the Data field of the Vault.
func (t *Vault) Encrypt(c *config.Config, store *store.Store) error {
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

		passphrase, err := GetPassphrase(c)
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

func (t *Vault) Write(fp string) error {
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
