package openpgp

import (
	"bytes"
	"fmt"
	"io/ioutil"

	"golang.org/x/crypto/openpgp"
	"golang.org/x/crypto/openpgp/armor"
)

type OpenPGPDecrypter struct {
	// Keys represents the collection of decryption keys
	// present in your gnupg secret ring
	Keys *openpgp.EntityList

	// passphrase is the passphrase to be used when
	// attempting to decrypt the provided bytes using
	// your gnupg secring keys
	passphrase string
}

// Decrypt reads up the openpgp encrypted data bytes from ed,
// decrypts them and returns the resulting plain data bytes as well
// as any potential errors.
func (od *OpenPGPDecrypter) Decrypt(ed []byte) ([]byte, error) {
	var armoredBlock *armor.Block
	var message *openpgp.MessageDetails
	var plain []byte
	var err error

	if ed == nil {
		return nil, nil
	}

	// Decode the OpenPGP armored block
	armoredBlock, err = armor.Decode(bytes.NewReader(ed))
	if err != nil {
		return nil, err
	}

	// Extract the message from the OpenPGP armored block
	message, err = openpgp.ReadMessage(armoredBlock.Body, od.Keys,
		func(keys []openpgp.Key, symmetric bool) ([]byte, error) {
			kp := []byte(od.passphrase)

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

// NewOpenPGPDecrypter builds a new OpenPGPDecrypter object
// from a gnupg secring file path and a passphrase.
// The returned object can then be used against OpenPGP
// encrypted bytes using the Decrypt method.
//
// See Decrypter interface.
func NewOpenPGPDecrypter(secRingPath string, passphrase string) (*OpenPGPDecrypter, error) {
	var dk *openpgp.EntityList
	var od *OpenPGPDecrypter
	var err error

	dk, err = ReadSecRing(secRingPath)
	if err != nil {
		return nil, err
	}

	od = &OpenPGPDecrypter{
		Keys:       dk,
		passphrase: passphrase,
	}

	return od, err
}
