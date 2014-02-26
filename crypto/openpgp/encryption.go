package openpgp

import (
	"bytes"
	"io"
	"log"
	"os"
	"strings"

	_ "crypto/ecdsa"
	_ "crypto/sha1"
	_ "crypto/sha256"
	_ "crypto/sha512"

	"code.google.com/p/go.crypto/openpgp"
	"code.google.com/p/go.crypto/openpgp/armor"
)

var encryptKeys openpgp.EntityList

func Encrypt(s string) []byte {
	buf := &bytes.Buffer{}

	wa, err := armor.Encode(buf, "PGP MESSAGE", nil)
	if err != nil {
		log.Fatalf("Can't make armor: %v", err)
	}

	w, err := openpgp.Encrypt(wa, encryptKeys, nil, nil, nil)
	if err != nil {
		log.Fatalf("Error encrypting: %v", err)
	}
	_, err = io.Copy(w, strings.NewReader(s))
	if err != nil {
		log.Fatalf("Error encrypting: %v", err)
	}

	w.Close()
	wa.Close()

	return buf.Bytes()
}

func InitEncryption(kr string, keyids []string) {
	f, err := os.Open(kr)
	if err != nil {
		log.Fatalf("Unable to open gnupg keyring: %v", err)
	}
	defer f.Close()

	kl, err := openpgp.ReadKeyRing(f)
	if err != nil {
		log.Fatalf("Unable to read from gnupg keyring: %v", err)
	}

	var hprefs, sprefs []uint8

	for _, keyId := range keyids {
		for _, entity := range kl {

			if isEntityKey(keyId, entity) {
				pi := primaryIdentity(entity)
				ss := pi.SelfSignature

				hprefs = intersectPreferences(hprefs, ss.PreferredHash)
				sprefs = intersectPreferences(sprefs, ss.PreferredSymmetric)
				encryptKeys = append(encryptKeys, entity)
			}
		}
	}

	if len(encryptKeys) != len(keyids) {
		log.Fatalf("Couldn't find all keys")
	}
	if len(hprefs) == 0 {
		log.Fatalf("No common hashes for encryption keys")
	}
	if len(sprefs) == 0 {
		log.Fatalf("No common symmetric ciphers for encryption keys")
	}
}

func isEntityKey(keyId string, e *openpgp.Entity) bool {
	if e.PrimaryKey.KeyIdShortString() == keyId {
		return true
	} else {
		for _, identity := range e.Identities {
			if identity.UserId.Email == keyId {
				return true
			}
		}
	}

	return false
}

func intersectPreferences(a []uint8, b []uint8) (intersection []uint8) {
	if a == nil {
		return b
	}

	var j int
	for _, v := range a {
		for _, v2 := range b {
			if v == v2 {
				a[j] = v
				j++
				break
			}
		}
	}

	return a[:j]
}

func primaryIdentity(e *openpgp.Entity) *openpgp.Identity {
	var firstIdentity *openpgp.Identity

	for _, ident := range e.Identities {
		if firstIdentity == nil {
			firstIdentity = ident
		}
		if ident.SelfSignature.IsPrimaryId != nil && *ident.SelfSignature.IsPrimaryId {
			return ident
		}
	}

	return firstIdentity
}
