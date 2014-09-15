package trousseau

import (
	"github.com/oleiade/trousseau/crypto/openpgp"
	"path"
	"os"
)


// Declare encryption types
type CryptoType int

const (
	SYMMETRIC_ENCRYPTION  CryptoType = 0
	ASYMMETRIC_ENCRYPTION CryptoType = 1
)

// Declare available encryption algorithms
type CryptoAlgorithm int

const (
	GPG_ENCRYPTION     CryptoAlgorithm = 0
	AES_256_ENCRYPTION CryptoAlgorithm = 1
)

// Gnupg variables
var GnupgHome = path.Join(os.Getenv("HOME"), ".gnupg")
var GnupgPubring func()string = func()string { return path.Join(GnupgHome, "pubring.gpg") }
var GnupgSecring func()string = func()string { return path.Join(GnupgHome, "secring.gpg") }

// DecryptAsymmetricPGP decrypts an OpenPGP message using GnuPG.
func DecryptAsymmetricPGP(encryptedData []byte, passphrase string) ([]byte, error) {
	// Decrypt store data
	decryptionKeys, err := openpgp.ReadSecRing(GnupgSecring())
	if err != nil {
		return nil, err
	}

	plainData, err := openpgp.Decrypt(encryptedData, decryptionKeys, passphrase)
	if err != nil {
		return nil, err
	}

	return plainData, nil
}

func EncryptAsymmetricPGP(plainData []byte, recipients []string) ([]byte, error) {
	encryptionKeys, err := openpgp.ReadPubRing(GnupgPubring(), recipients)
	if err != nil {
		return nil, err
	}

	encData, err := openpgp.Encrypt(plainData, encryptionKeys)
	if err != nil {
		return nil, err
	}

	return encData, nil
}
