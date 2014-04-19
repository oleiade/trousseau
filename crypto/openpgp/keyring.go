package openpgp

import (
	"code.google.com/p/go.crypto/openpgp"
	"fmt"
	"os"
)

func ReadPubRing(path string, keyIds []string) (*openpgp.EntityList, error) {
	var pubKeys *openpgp.EntityList
	var matchedKeys openpgp.EntityList
	var hprefs, sprefs []uint8
	var err error

	pubKeys, err = ReadKeyRing(path)
	if err != nil {
		return nil, err
	}

	for _, keyId := range keyIds {
		for _, entity := range *pubKeys {

			if isEntityKey(keyId, entity) {
				pi := primaryIdentity(entity)
				ss := pi.SelfSignature

				hprefs = intersectPreferences(hprefs, ss.PreferredHash)
				sprefs = intersectPreferences(sprefs, ss.PreferredSymmetric)
				matchedKeys = append(matchedKeys, entity)
			}
		}
	}

	if len(matchedKeys) != len(keyIds) {
		fmt.Println("HERE")
		return nil, fmt.Errorf("Couldn't find all keys")
	}
	if len(hprefs) == 0 {
		return nil, fmt.Errorf("No common hashes for encryption keys")
	}
	if len(sprefs) == 0 {
		return nil, fmt.Errorf("No common symmetric ciphers for encryption keys")
	}

	return &matchedKeys, nil
}

func ReadSecRing(path string) (*openpgp.EntityList, error) {
	return ReadKeyRing(path)
}

func ReadKeyRing(path string) (*openpgp.EntityList, error) {
	var keys openpgp.EntityList
	var err error

	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("Unable to open gnupg keyring: %v", err)
	}
	defer f.Close()

	keys, err = openpgp.ReadKeyRing(f)
	if err != nil {
		return nil, fmt.Errorf("Unable to read from gnupg keyring: %v", err)
	}

	return &keys, nil
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
