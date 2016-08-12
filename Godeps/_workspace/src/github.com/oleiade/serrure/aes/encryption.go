package aes

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"io"
)

// AES256Encrypter implements the Encrypter interface.
// Provided a AES256Key object it exposes a Encrypt method to
// encrypt provided plain bytes using AES256 algorithm.
type AES256Encrypter struct {
	Key *AES256Key
}

// Encrypt reads up plain data bytes contained in pd, encrypts
// them using AES256 encryption algorithm, and returns the
// resulting bytes as well as any potential errors.
func (a *AES256Encrypter) Encrypt(pd []byte) ([]byte, error) {
	var ciphertext []byte

	block, err := aes.NewCipher(a.Key.key)
	if err != nil {
		return nil, err
	}

	ciphertext = make([]byte, aes.BlockSize+len(pd))
	iv := ciphertext[:aes.BlockSize]
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		return nil, err
	}

	stream := cipher.NewCFBEncrypter(block, iv)
	stream.XORKeyStream(ciphertext[aes.BlockSize:], pd)
	ciphertext = PrependSalt(a.Key.salt, ciphertext)

	return ciphertext, nil
}

// NewAES256Encrypter builds a new AES256Encrypter object
// from provided passphrase and salt.
// The returned object can then be used against byte slices
// to encrypt them with the AES256 encryption algorithm using
// the Encrypt method.
//
// See Encrypter interface.
func NewAES256Encrypter(Passphrase string, salt []byte) (*AES256Encrypter, error) {
	var k *AES256Key
	var ae *AES256Encrypter
	var err error

	k, err = MakeAES256Key(Passphrase, salt)
	if err != nil {
		return nil, err
	}

	ae = &AES256Encrypter{
		Key: k,
	}

	return ae, err
}
