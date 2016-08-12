package aes

import (
	"crypto/aes"
	"crypto/cipher"
	"errors"
)

// AES256Decrypter implements the Decrypter interface.
// Provided a Passphrase it exposes a Decrypt method to
// read the content of AES256 encrypted bytes.
type AES256Decrypter struct {
	// Passphrase to be used to decrypt the AES256 ciphered blocks
	Passphrase string
}

// Decrypt reads up the AES256 encrypted data bytes from ed,
// decrypts them and returns the resulting plain data bytes as well
// as any potential errors.
func (a *AES256Decrypter) Decrypt(ed []byte) ([]byte, error) {
	var aesKey *AES256Key
	var ciphertext []byte
	var err error

	ciphertext, aesKey, err = parseMsg(a.Passphrase, ed)
	if err != nil {
		return nil, err
	}

	block, err := aes.NewCipher(aesKey.key)
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

// extractMsg extracts ciphertext from message
func extractMsg(ciphertext []byte) ([]byte, error) {
	if len(ciphertext) < SALT_SIZE+aes.BlockSize {
		return nil, errors.New("Ciphertext too short")
	}

	return ciphertext[SALT_SIZE:], nil
}

func parseMsg(Passphrase string, msg []byte) ([]byte, *AES256Key, error) {
	salt, err := ExtractSalt(msg)
	if err != nil {
		return nil, nil, err
	}

	ciphertext, err := extractMsg(msg)
	if err != nil {
		return nil, nil, err
	}

	aeskey, err := MakeAES256Key(Passphrase, salt)
	if err != nil {
		return nil, nil, err
	}

	return ciphertext, aeskey, nil
}

// NewAES256Decrypter builds a new AES256Decrypter object
// from Passphrase. The returned object can then be used
// against AES256 encrypted bytes using this Passphrase
// using the Decrypt method.
//
// See Decrypter interface.
func NewAES256Decrypter(p string) *AES256Decrypter {
	return &AES256Decrypter{
		Passphrase: p,
	}
}
