package openpgp

import (
	"bytes"
	"code.google.com/p/go.crypto/openpgp"
	"code.google.com/p/go.crypto/openpgp/armor"
	"fmt"
	"io"
	"io/ioutil"
	"log"
)

// Encrypt the provided bytes for the provided encryption
// keys recipients. Returns the encrypted content bytes.
func Encrypt(d []byte, encryptionKeys *openpgp.EntityList) ([]byte, error) {
	var buffer *bytes.Buffer = &bytes.Buffer{}
	var armoredWriter io.WriteCloser
	var cipheredWriter io.WriteCloser
	var err error

	// Create an openpgp armored cipher writer pointing on our
	// buffer
	armoredWriter , err = armor.Encode(buffer, "PGP MESSAGE", nil)
	if err != nil {
		NewPgpError(ERR_ENCRYPTION_ENCODING, fmt.Sprintf("Can't make armor: %v", err))
	}

	// Create an encrypted writer using the provided encryption keys
	cipheredWriter, err = openpgp.Encrypt(armoredWriter, *encryptionKeys, nil, nil, nil)
	if err != nil {
		NewPgpError(ERR_ENCRYPTION_ENCRYPT, fmt.Sprintf("Error encrypting: %v", err))
	}

	// Write (encrypts on the fly) the provided bytes to
	// cipheredWriter
	_, err = cipheredWriter.Write(d)
	if err != nil {
		log.Fatalf("Error copying encrypted content: %v", err)
	}

	cipheredWriter.Close()
	armoredWriter.Close()

	return buffer.Bytes(), nil
}

// Decrypt tries to decrypt an OpenPGP armored block using the provided decryption keys
// and passphrase. If succesfull the plain content of the block is returned as []byte.
func Decrypt(d []byte, decryptionKeys *openpgp.EntityList, passphrase string) ([]byte, error) {
	var armoredBlock *armor.Block
	var message *openpgp.MessageDetails
	var plain []byte
	var err error

	if d == nil {
		return nil, nil
	}

	// Decode the OpenPGP armored block
	armoredBlock, err = armor.Decode(bytes.NewReader(d))
	if err != nil {
		return nil, err
	}

	// Extract the message from the OpenPGP armored block
	message, err = openpgp.ReadMessage(armoredBlock.Body, decryptionKeys,
		func(keys []openpgp.Key, symmetric bool) ([]byte, error) {
			kp := []byte(passphrase)

			if symmetric {
				return kp, nil
			}

			for _, k := range keys {
				err := k.PrivateKey.Decrypt(kp)
				if err == nil {
					// If no error were returned, we could succesfully
					// decrypt the message using the provided private key
					return nil, nil
				}
			}

			return nil, fmt.Errorf("Unable to decrypt trousseau data store. " +
				"Invalid passphrase supplied.")
		},
		nil)
	if err != nil {
		return nil, fmt.Errorf("unable to decrypt trousseau data store. " +
							   "No private key able to decrypt it found in your keyring.")
	}

	// Read the plain message bytes
	plain, err = ioutil.ReadAll(message.UnverifiedBody)
	if err != nil {
		return nil, err
	}

	return plain, err
}
