package aes

import (
	"crypto/aes"
	"crypto/rand"
	"errors"
	"io"
)

const SALT_SIZE = 16

// GenerateSalte securely generates a len(SALT_SIZE) byte salt
func GenerateSalt() ([]byte, error) {
	var salt []byte = make([]byte, SALT_SIZE)

	if _, err := io.ReadFull(rand.Reader, salt); err != nil {
		return nil, err
	}

	return salt, nil
}

// ExtractSalt separates salt and ciphertext
func ExtractSalt(ciphertext []byte) ([]byte, error) {
	if len(ciphertext) < SALT_SIZE+aes.BlockSize { //replace these with actual values
		return nil, errors.New("Ciphertext too short")
	}

	return ciphertext[:SALT_SIZE], nil
}

// PrependSalt prepends salt to the ciphertext
func PrependSalt(salt, ciphertext []byte) []byte {
	var msg []byte = make([]byte, len(salt)+len(ciphertext))

	for i := 0; i < len(salt)+len(ciphertext); i++ {
		if i >= len(salt) {
			msg[i] = ciphertext[i-len(salt)]
		} else {
			msg[i] = salt[i]
		}
	}

	return msg
}
