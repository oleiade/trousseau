package aes

import (
	"crypto/aes"
	"errors"

	"code.google.com/p/go.crypto/scrypt"
)

type AES256Key struct {
	key  []byte
	salt []byte
}

// Generate a new AES256 key from a key and salt
func NewAes256Key(key, salt []byte) AES256Key {
	a := AES256Key{key, salt}
	return a
}

// make an AES key. Pass nil as salt if you want to generate a new one
// otherwise pass the salt from the message and you will get the key
// will use scrypt to make it semi secure
func MakeAES256Key(passphrase string, salt []byte) (*AES256Key, error) {
	b := []byte(passphrase)
	var err error = nil
	if salt == nil {
		salt, err = GenerateSalt()
		if err != nil {
			return nil, err
		}
	} else {
		if len(salt) != saltSize {
			return nil, errors.New("Salt is not the correct size")
		}
	}
	key, err := scrypt.Key(b, salt, 65536, aes.BlockSize, 1, 32)
	if err != nil {
		return nil, err
	}
	aeskey := NewAes256Key(key, salt)

	return &aeskey, nil
}
