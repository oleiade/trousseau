package openpgp

import (
	"bytes"
	"fmt"
	"io"

	"golang.org/x/crypto/openpgp"
	"golang.org/x/crypto/openpgp/armor"
)

// OpenPGPEncrypter implements the Encrypter interface.
// Provided an *openpgp.EntityList object it exposes an Encrypt method to
// encrypt provided plain bytes using OpenPGP algorithm.
type OpenPGPEncrypter struct {
	Keys *openpgp.EntityList
}

// Encrypt reads up plain data bytes contained in pd, encrypts
// them using OpenPGP encryption algorithm, and returns the
// resulting bytes as well as any potential errors.
func (oe *OpenPGPEncrypter) Encrypt(pd []byte) ([]byte, error) {
	var buffer *bytes.Buffer = &bytes.Buffer{}
	var armoredWriter io.WriteCloser
	var cipheredWriter io.WriteCloser
	var err error

	// Create an openpgp armored cipher writer pointing on our
	// buffer
	armoredWriter, err = armor.Encode(buffer, "PGP MESSAGE", nil)
	if err != nil {
		return nil, NewOpenPGPError(
			ERR_ENCRYPTION_ENCODING,
			fmt.Sprintf("Can't make armor: %v", err),
		)
	}

	// Create an encrypted writer using the provided encryption keys
	cipheredWriter, err = openpgp.Encrypt(armoredWriter, *oe.Keys, nil, nil, nil)
	if err != nil {
		return nil, NewOpenPGPError(
			ERR_ENCRYPTION_ENCRYPT,
			fmt.Sprintf("Error encrypting: %v", err),
		)
	}

	// Write (encrypts on the fly) the provided bytes to
	// cipheredWriter
	_, err = cipheredWriter.Write(pd)
	if err != nil {
		return nil, NewOpenPGPError(
			ERR_ENCRYPTION_ENCRYPT,
			fmt.Sprintf("Error copying encrypted content: %v", err),
		)
	}

	cipheredWriter.Close()
	armoredWriter.Close()

	return buffer.Bytes(), nil
}

// NewOpenPGPEncrypter builds a new OpenPGPEncrypter object
// from provided gnupg pubring file path and a list of recipients.
// The returned object can then be used against byte slices to encrypt
// them with the OpenPGP encryption algorithm using
// the Encrypt method.
//
// See Encrypter interface.
func NewOpenPGPEncrypter(pubRingPath string, recipients []string) (*OpenPGPEncrypter, error) {
	var ek *openpgp.EntityList
	var oe *OpenPGPEncrypter
	var err error

	ek, err = ReadPubRing(pubRingPath, recipients)
	if err != nil {
		return nil, err
	}

	oe = &OpenPGPEncrypter{
		Keys: ek,
	}

	return oe, err
}
