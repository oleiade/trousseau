package aes

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"errors"
	"io"
)

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

func Decrypt(passphrase string, msg []byte) ([]byte, error) {
	ciphertext, key, err := ParseMsg(passphrase, msg)
	if err != nil {
		return nil, err
	}
	plaintext, err := DecryptAES256(*key, ciphertext)
	if err != nil {
		return nil, err
	}
	return plaintext, nil
}

func Encrypt(passphrase string, plainData []byte) ([]byte, error) {
	key, err := MakeAES256Key(passphrase, nil)
	if err != nil {
		return nil, err
	}
	ciphertext, err := EncryptAES256(*key, plainData)
	if err != nil {
		return nil, err
	}
	return ciphertext, nil
}
