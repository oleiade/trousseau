package openpgp

import (
	"bytes"
	"code.google.com/p/go.crypto/openpgp"
	"code.google.com/p/go.crypto/openpgp/armor"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"strings"
)

func Encrypt(encryptionKeys *openpgp.EntityList, s string) []byte {
	buf := &bytes.Buffer{}

	wa, err := armor.Encode(buf, "PGP MESSAGE", nil)
	if err != nil {
		NewPgpError(ERR_ENCRYPTION_ENCODING, fmt.Sprintf("Can't make armor: %v", err))
	}

	w, err := openpgp.Encrypt(wa, *encryptionKeys, nil, nil, nil)
	if err != nil {
		NewPgpError(ERR_ENCRYPTION_ENCRYPT, fmt.Sprintf("Error encrypting: %v", err))
	}

	_, err = io.Copy(w, strings.NewReader(s))
	if err != nil {
		log.Fatalf("Error copying encrypted content: %v", err)
	}

	w.Close()
	wa.Close()

	return buf.Bytes()
}

func Decrypt(decryptionKeys *openpgp.EntityList, s, passphrase string) ([]byte, error) {
	if s == "" {
		return nil, nil
	}

	armorBlock, err := armor.Decode(strings.NewReader(s))
	if err != nil {
		return nil, err
	}

	d, err := openpgp.ReadMessage(armorBlock.Body, decryptionKeys,
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

	bytes, err := ioutil.ReadAll(d.UnverifiedBody)
	return bytes, err
}
