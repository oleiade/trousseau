package crypto

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"errors"
	"fmt"
	"io"

	"golang.org/x/crypto/scrypt"
)

type AES256Service struct {
	Cipher     *AES256Cipher
	Passphrase string
}

// NewAES256Service instantiates a new AES256Service
func NewAES256Service(passphrase string) (*AES256Service, error) {
	salt, err := GenerateSalt(SaltSize)
	if err != nil {
		return nil, err
	}

	cipher, err := NewAES256Cipher(passphrase, salt)
	if err != nil {
		return nil, err
	}

	return &AES256Service{Cipher: cipher, Passphrase: passphrase}, nil
}

// Encrypt reads up plain from the Reader, encrypts
// it using AES256 CFB encryption algorithm, and writes the
// result to the Writer.
func (a *AES256Service) Encrypt(plain []byte) ([]byte, error) {
	ciphertext := make([]byte, aes.BlockSize+len(plain))
	iv := ciphertext[:aes.BlockSize]
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		return nil, err
	}

	stream := cipher.NewCFBEncrypter(a.Cipher.Block, iv)
	stream.XORKeyStream(ciphertext[aes.BlockSize:], plain)
	ciphertext = prependSalt(a.Cipher.Salt, ciphertext)

	return ciphertext, nil
}

// Decrypt reads up the AES256 encrypted data bytes from ed,
// decrypts them and returns the resulting plain data bytes as well
// as any potential errors.
func (a *AES256Service) Decrypt(encrypted []byte) ([]byte, error) {
	if len(encrypted) < SaltSize+aes.BlockSize {
		return nil, fmt.Errorf("ciphertext is too short")
	}

	// Extract the salt that was prepended during encryption and
	// re-derive the key from it, rather than using the service's
	// randomly generated salt which differs from the original.
	salt := encrypted[:SaltSize]
	ciphertext := encrypted[SaltSize:]
	if len(ciphertext) < aes.BlockSize {
		return nil, fmt.Errorf("ciphertext is too short")
	}

	key, err := scrypt.Key([]byte(a.Passphrase), salt, KeyCost, aes.BlockSize, 1, 32)
	if err != nil {
		return nil, err
	}
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}

	iv := ciphertext[:aes.BlockSize]
	ciphertext = ciphertext[aes.BlockSize:]
	stream := cipher.NewCFBDecrypter(block, iv)
	stream.XORKeyStream(ciphertext, ciphertext)

	return ciphertext, nil
}

const KeyCost int = 65536
const SaltSize int = 16

type AES256Cipher struct {
	Key   []byte
	Salt  []byte
	Block cipher.Block
}

// NewAES256Cipher generates a new AES256 cipher from a key and salt
func NewAES256Cipher(passphrase string, salt []byte) (*AES256Cipher, error) {
	b := []byte(passphrase)

	if len(salt) != SaltSize {
		return nil, errors.New("Salt is not the correct size")
	}

	key, err := scrypt.Key(b, salt, 65536, aes.BlockSize, 1, 32)
	if err != nil {
		return nil, err
	}

	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}

	return &AES256Cipher{key, salt, block}, nil
}

// GenerateSalt securely generates a Salt string
func GenerateSalt(size int) ([]byte, error) {
	salt := make([]byte, size)

	if _, err := io.ReadFull(rand.Reader, salt); err != nil {
		return nil, err
	}

	return salt, nil
}

// PrependSalt prepends salt to the ciphertext
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
