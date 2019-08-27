package trousseau

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"

	"github.com/mcuadros/go-defaults"
	"github.com/oleiade/trousseau/internal/config"
)

const TROUSSEAU_VERSION = "0.4.1"

// Vault holds the public and encrypted trousseau data store.
type Vault struct {
	// Crypto public configuration attributes
	CryptoType      CryptoType      `json:"crypto_type" default:"asymmetric"`
	CryptoAlgorithm CryptoAlgorithm `json:"crypto_algorithm" default:"gpg"`

	// Encrypted data private attribute
	Encrypted []byte `json:"_data"`

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

// Unlock deciphers the encrypted part of the Vault, and returns the
// unencrypted Store.
func (v *Vault) Unlock(c *config.Config) (*SecretStore, error) {
	passphrase, err := GetPassphrase(c)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	ss := NewSecretStore()
	err = ss.Decrypt(bytes.NewReader(v.Encrypted), v.CryptoAlgorithm, passphrase)
	if err != nil {
		return nil, err
	}

	return ss, nil
}

// Lock encrypts a Store into the Data field of the Vault.
func (v *Vault) Lock(c *config.Config, ss *SecretStore) error {
	passphrase, err := GetPassphrase(c)
	if err != nil {
		return err
	}

	ciphered, err := ss.Encrypt(v.CryptoAlgorithm, passphrase)
	if err != nil {
		return err
	}

	v.Encrypted = ciphered

	return nil
}

// ReadVault loads a Vault from a slice of bytes.
// It returns an error if the loaded Vault's version is not compatible
// with the version of trousseau currently being used
func ReadVault(r io.Reader) (*Vault, error) {
	data, err := ioutil.ReadAll(r)
	if err != nil {
		return nil, err
	}

	var vault *Vault
	err = json.Unmarshal(data, &vault)
	if err != nil {
		// Check if the content of the file matches with a legacy
		// data store file format. Raise a proper error accordingly.
		contentVersion := DiscoverVersion(data, VersionDiscoverClosures)
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

// Dump encodes the vault to JSON and writes it to w.
func (v *Vault) Dump(w io.Writer) error {
	return json.NewEncoder(w).Encode(v)
}
