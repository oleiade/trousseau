package aes

import (
	"crypto/aes"
	"crypto/rand"
	"errors"
	"io"
)

// Securely generate a 8 byte salt
func GenerateSalt() ([]byte, error) {
	salt := make([]byte, saltSize)
	if _, err := io.ReadFull(rand.Reader, salt); err != nil {
		return nil, err
	}
	return salt, nil
}

// helper functions to separate salt and message
func ExtractSalt(ciphertext []byte) ([]byte, error) {
	if len(ciphertext) < saltSize+aes.BlockSize { //replace these with actual values
		return nil, errors.New("Ciphertext too short")
	}

	return ciphertext[:saltSize], nil
}

// get ciphertext from message
func ExtractMsg(ciphertext []byte) ([]byte, error) {
	if len(ciphertext) < saltSize+aes.BlockSize {
		return nil, errors.New("Ciphertext too short")
	}

	return ciphertext[saltSize:], nil
}

func ParseMsg(passphrase string, msg []byte) ([]byte, *AES256Key, error) {
	salt, err := ExtractSalt(msg)
	if err != nil {
		return nil, nil, err
	}
	ciphertext, err := ExtractMsg(msg)
	if err != nil {
		return nil, nil, err
	}
	aeskey, err := MakeAES256Key(passphrase, salt)
	if err != nil {
		return nil, nil, err
	}

	return ciphertext, aeskey, nil
}

// Prepend salt to the message
func prependSalt(salt, ciphertext []byte) []byte {
	msg := make([]byte, len(salt)+len(ciphertext))
	for i := 0; i < len(salt)+len(ciphertext); i++ {
		if i >= len(salt) {
			msg[i] = ciphertext[i-len(salt)]
		} else {
			msg[i] = salt[i]
		}
	}
	return msg
}
