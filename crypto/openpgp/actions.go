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

	// Create an encrypted writer using
	cipheredWriter, err = openpgp.Encrypt(armoredWriter, *encryptionKeys, nil, nil, nil)
	if err != nil {
		NewPgpError(ERR_ENCRYPTION_ENCRYPT, fmt.Sprintf("Error encrypting: %v", err))
	}

	_, err = cipheredWriter.Write(d)
	if err != nil {
		log.Fatalf("Error copying encrypted content: %v", err)
	}

	cipheredWriter.Close()
	armoredWriter.Close()

	return buffer.Bytes(), nil
}

func Decrypt(d []byte, decryptionKeys *openpgp.EntityList, passphrase string) ([]byte, error) {
	var armoredBlock *armor.Block
	var message *openpgp.MessageDetails
	var plain []byte
	var err error

	if d == nil {
		return nil, nil
	}

	armoredBlock, err = armor.Decode(bytes.NewReader(d))
	if err != nil {
		return nil, err
	}

	message, err = openpgp.ReadMessage(armoredBlock.Body, decryptionKeys,
		func(keys []openpgp.Key, symmetric bool) ([]byte, error) {
			kp := []byte(passphrase)

			if symmetric {
				return kp, nil
			}

			for _, k := range keys {
				err := k.PrivateKey.Decrypt(kp)
				if err == nil {
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

	plain, err = ioutil.ReadAll(message.UnverifiedBody)

	return plain, err
}
