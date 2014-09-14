package trousseau

import (
	"encoding/json"
	"fmt"
	"github.com/oleiade/trousseau/crypto/openpgp"
	"sort"
)

type VersionMatcher func([]byte) bool
type UpgradeClosure func([]byte) ([]byte, error)

var UpgradeClosures map[string]UpgradeClosure = map[string]UpgradeClosure{
	"0.3.0": upgradeZeroDotThreeToNext,
}

var VersionDiscoverClosures map[string]VersionMatcher = map[string]VersionMatcher{
	"0.3.0": isVersionZeroDotThreeDotZero,
	"0.3.1": isVersionZeroDotThreeDotOne,
}

func UpgradeFrom(startVersion string, d []byte, mapping map[string]UpgradeClosure) ([]byte, error) {
	var versions []string
	var out []byte = d
	var err error

	for version, _ := range mapping {
		if version >= startVersion {
			versions = append(versions, version)
		}
	}
	sort.Strings(versions)

	for idx, version := range versions {
		var versionRepr string

		if idx == (len(versions) - 1) {
			versionRepr = TROUSSEAU_VERSION
		} else {
			versionRepr = version
		}

		upgradeClosure := mapping[version]
		out, err = upgradeClosure(out)
		if err != nil {
			return nil, fmt.Errorf("Upgrading trousseau data store to version %s: failure\nReason: %s", versionRepr, err.Error())
		}
	}

	return out, nil
}

func DiscoverVersion(d []byte, mapping map[string]VersionMatcher) string {
	var versions []string

	for version, _ := range mapping {
		versions = append(versions, version)
	}
	sort.Strings(versions)

	for _, version := range versions {
		if mapping[version](d) == true {
			return version
		}
	}

	return ""
}

func isVersionZeroDotThreeDotZero(d []byte) bool {
	if len(d) >= len(openpgp.PGP_MESSAGE_HEADER) &&
		string(d[0:len(openpgp.PGP_MESSAGE_HEADER)]) == openpgp.PGP_MESSAGE_HEADER {
		return true
	}

	return false
}

func isVersionZeroDotThreeDotOne(d []byte) bool {
	var zeroDotFourStore map[string]interface{} = make(map[string]interface{})
	var zeroDotFourKeys []string = []string{"crypto_algorithm", "crypto_type", "_data"}

	err := json.Unmarshal(d, &zeroDotFourStore)
	if err != nil {
		return false
	}

	for _, key := range zeroDotFourKeys {
		_, in := zeroDotFourStore[key]
		if !in {
			return false
		}
	}

	return true
}

func upgradeZeroDotThreeToNext(d []byte) ([]byte, error) {
	var err error

	// Assert input data are in the expected version format
	validVersion := isVersionZeroDotThreeDotZero(d)
	if !validVersion {
		return nil, fmt.Errorf("Provided input data not matching version 0.3 format")
	}

	// Declaring and instanciating a type matching
	// the 0.3 version store format
	legacyStore := struct {
		Meta map[string]interface{} `json:"_meta"`
		Data map[string]interface{} `json:"data"`
	}{
		Meta: make(map[string]interface{}),
		Data: make(map[string]interface{}),
	}

	// Retrieve secret ring keys from openpgp
	decryptionKeys, err := openpgp.ReadSecRing(openpgp.SecringFile)
	if err != nil {
		return nil, err
	}

	// Decrypt store version 0.3 (aka legacy)
	plainData, err := openpgp.Decrypt(decryptionKeys, string(d), GetPassphrase())
	if err != nil {
		return nil, err
	}

	// Unmarshal it's content into the legacyStore
	err = json.Unmarshal(plainData, &legacyStore)
	if err != nil {
		return nil, err
	}

	// Declaring and instanciating a type matching
	// the 0.4 version store format so we can inject the
	// legacy data  into it
	newStore := struct {
		Meta map[string]interface{} `json:"meta"`
		Data map[string]interface{} `json:"store"`
	}{
		Meta: legacyStore.Meta,
		Data: legacyStore.Data,
	}

	// Encode it in json
	newStoreData, err := json.Marshal(newStore)
	if err != nil {
		return nil, err
	}

	// Retrieve legacyStore recipients
	var recipients []string
	for _, r := range legacyStore.Meta["recipients"].([]interface{}) {
		recipients = append(recipients, r.(string))
	}

	// Read the public openpgp ring to retrieve the recipients public keys
	encryptionKeys, err := openpgp.ReadPubRing(openpgp.PubringFile, recipients)
	if err != nil {
		return nil, err
	}

	// Encrypt the encoded newStore content
	encryptedData, err := openpgp.Encrypt(newStoreData, encryptionKeys)
	if err != nil {
		return nil, err
	}

	// Declaring and instanciating a type matching
	// the 0.4 version trousseau data store format
	// so we can inject the encrypted store into it
	newTrousseau := struct {
		CryptoAlgorithm CryptoAlgorithm `json:"crypto_algorithm"`
		CryptoType      CryptoType      `json:"crypto_type"`
		Data            []byte          `json:"_data"`
	}{
		CryptoAlgorithm: GPG_ENCRYPTION,
		CryptoType:      ASYMMETRIC_ENCRYPTION,
		Data:            encryptedData,
	}

	// Encode the new trousseau data store
	trousseau, err := json.Marshal(newTrousseau)
	if err != nil {
		return nil, err
	}

	return trousseau, nil
}
