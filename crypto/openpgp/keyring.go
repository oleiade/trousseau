package openpgp

import (
	"code.google.com/p/go.crypto/openpgp"
	"fmt"
	"os"
	"strings"
)

func ReadPubRing(path string, keyIds []string) (*openpgp.EntityList, error) {
	var pubKeys *openpgp.EntityList
	var matchedKeys openpgp.EntityList
	var unmatchedKeyIds []string
	var hprefs, sprefs []uint8
	var err error

	pubKeys, err = ReadKeyRing(path)
	if err != nil {
		return nil, err
	}

	// For each provided key ids, check if it exists
	// through the gnupg pub ring
	for _, keyId := range keyIds {
		var matched bool = false

		for _, entity := range *pubKeys {
			if isEntityKey(keyId, entity) {
				pi := primaryIdentity(entity)
				ss := pi.SelfSignature

				hprefs = intersectPreferences(hprefs, ss.PreferredHash)
				sprefs = intersectPreferences(sprefs, ss.PreferredSymmetric)
				matchedKeys = append(matchedKeys, entity)
				matched = true
			}
		}

		if matched == false {
			unmatchedKeyIds = append(unmatchedKeyIds, keyId)
		}

	}

	if len(unmatchedKeyIds) != 0 {
		errMsg := fmt.Sprintf("The following keys could not be found "+
			"in the public keyring: %s", strings.Join(unmatchedKeyIds, ", "))
		return nil, NewPgpError(ERR_DECRYPTION_KEYS, errMsg)
	}
	if len(hprefs) == 0 {
		return nil, NewPgpError(ERR_DECRYPTION_HASHES, "No common hashes for encryption keys")
	}
	if len(sprefs) == 0 {
		return nil, NewPgpError(ERR_DECRYPTION_CIPHERS, "No common symmetric ciphers for encryption keys")
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
		return nil, NewPgpError(ERR_KEYRING, fmt.Sprintf("Unable to open gnupg keyring: %v", err))
	}
	defer f.Close()

	keys, err = openpgp.ReadKeyRing(f)
	if err != nil {
		return nil, NewPgpError(ERR_KEYRING, fmt.Sprintf("Unable to read from gnupg keyring: %v", err))
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

func UserIds() []string {
	var userIds []string

	return userIds
}
