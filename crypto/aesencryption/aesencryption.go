package aesencryption

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"errors"
	"io"

	"code.google.com/p/go.crypto/scrypt"
)

var saltSize = 16

type AES256Key struct {
	key  []byte
	salt []byte
}

// Generate a new AES256 key from a key and salt
func NewAes256Key(key, salt []byte) AES256Key {
	a := AES256Key{key, salt}
	return a
}

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

// Encrypt AES256
func EncryptAES256(k AES256Key, plainData []byte) ([]byte, error) {
	block, err := aes.NewCipher(k.key)
	if err != nil {
		return nil, err
	}
	ciphertext := make([]byte, aes.BlockSize+len(plainData))
	iv := ciphertext[:aes.BlockSize]
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		return nil, err
	}
	stream := cipher.NewCFBEncrypter(block, iv)
	stream.XORKeyStream(ciphertext[aes.BlockSize:], plainData)
	ciphertext = prependSalt(k.salt, ciphertext)
	return ciphertext, nil
}

// Decrypt AES256
func DecryptAES256(k AES256Key, ciphertext []byte) ([]byte, error) {
	block, err := aes.NewCipher(k.key)
	if err != nil {
		return nil, err
	}
	if len(ciphertext) < aes.BlockSize {
		return nil, errors.New("Ciphertext too short")
	}
	iv := ciphertext[:aes.BlockSize]
	ciphertext = ciphertext[aes.BlockSize:]
	stream := cipher.NewCFBDecrypter(block, iv)
	stream.XORKeyStream(ciphertext, ciphertext)
	return ciphertext, nil
}
