package trousseau

import (
	"encoding/json"
	"fmt"
	"io"
	"time"

	"github.com/oleiade/serrure/openpgp"
	"github.com/oleiade/trousseau/internal/crypto"
)

type SecretStore struct {
	Metadata *Metadata         `json:"metadata"`
	Data     map[string]string `json:"data"`
}

func NewSecretStore() *SecretStore {
	return &SecretStore{
		Metadata: NewMetadata(),
		Data:     make(map[string]string),
	}
}

func (ss *SecretStore) Encrypt(algo CryptoAlgorithm, passphrase string) ([]byte, error) {
	data, err := json.Marshal(ss)
	if err != nil {
		return nil, err
	}

	switch algo {
	case GPG_ENCRYPTION:
		encrypter, err := openpgp.NewOpenPGPEncrypter(GnupgPubring(), ss.Metadata.Recipients)
		if err != nil {
			return nil, err
		}

		return encrypter.Encrypt(data)
	case AES_256_ENCRYPTION:
		encrypter, err := crypto.NewAES256Service(passphrase)
		if err != nil {
			return nil, err
		}

		return encrypter.Encrypt(data)
	default:
		return nil, fmt.Errorf("Invalid encryption method provided")
	}
}

func (ss *SecretStore) Decrypt(r io.Reader, algo CryptoAlgorithm, passphrase string) error {
	switch algo {
	case GPG_ENCRYPTION:
		decrypter, err := openpgp.NewOpenPGPDecrypter(GnupgSecring(), passphrase)
		if err != nil {
			return err
		}

		ciphered, err := io.ReadAll(r)
		if err != nil {
			return err
		}

		plain, err := decrypter.Decrypt(ciphered)
		if err != nil {
			return err
		}

		err = json.Unmarshal(plain, ss)
		if err != nil {
			return err
		}
	case AES_256_ENCRYPTION:
		decrypter, err := crypto.NewAES256Service(passphrase)
		if err != nil {
			return err
		}

		ciphered, err := io.ReadAll(r)
		if err != nil {
			return err
		}

		plain, err := decrypter.Decrypt(ciphered)
		if err != nil {
			return err
		}

		err = json.Unmarshal(plain, ss)
		if err != nil {
			return err
		}
	default:
		return fmt.Errorf("Invalid encryption method provided")
	}

	return nil
}

type Metadata struct {
	Version        string   `json:"version"`
	CreatedAt      string   `json:"created_at"`
	LastModifiedAt string   `json:"last_modified_at"`
	Recipients     []string `json:"recipients"`
}

func NewMetadata() *Metadata {
	return &Metadata{
		Version:        TROUSSEAU_VERSION,
		CreatedAt:      time.Now().String(),
		LastModifiedAt: time.Now().String(),
		Recipients:     []string{},
	}
}

func (m *Metadata) Update(version string, recipients []string) {
	m.Version = version
	m.LastModifiedAt = time.Now().String()
	m.Recipients = recipients
}
