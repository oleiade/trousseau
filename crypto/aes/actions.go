package aes

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"io"
)

// Encrypt AES256
func EncryptAES256(k AES256Key, plainData []byte) ([]byte, error) {
	block, err := aes.NewCipher(k.key)
	if err != nil {
		return nil, err
	}
	a, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}
	nonce := make([]byte, a.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return nil, err
	}
	ciphertext := a.Seal(nil, nonce, plainData, nil)
	ciphertext = prependSalt(k.salt, prependSalt(nonce, ciphertext))
	return ciphertext, nil
}

// Decrypt AES256
func DecryptAES256(k AES256Key, ciphertext []byte) ([]byte, error) {
	block, err := aes.NewCipher(k.key)
	if err != nil {
		return nil, err
	}
	a, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}
	nonce := ciphertext[:a.NonceSize()]
	ct := ciphertext[a.NonceSize():]
	pt, err := a.Open(nil, nonce, ct, nil)
	if err != nil {
		return nil, err
	}
	return pt, nil
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
