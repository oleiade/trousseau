package trousseau

import (
	"os"
	"encoding/json"
	"github.com/oleiade/trousseau/crypto/openpgp"
	"sort"
)

type VersionMatcher func([]byte)bool
type UpgradeClosure func(*os.File, *os.File)error

var upgradeClosures map[string]UpgradeClosure = map[string]UpgradeClosure {
	"0.3.0": nil,
	"0.4.0": nil,
}

var versionDiscoverClosures map[string]VersionMatcher = map[string]VersionMatcher {
	"0.3.0": isVersionZeroDotThree,
	"0.4.0": isVersionZeroDotFour,
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

func isVersionZeroDotThree(d []byte) bool {
	if len(d)>= len(openpgp.PGP_MESSAGE_HEADER) &&
	   string(d[0:len(openpgp.PGP_MESSAGE_HEADER)]) == openpgp.PGP_MESSAGE_HEADER {
		return true
	}

	return false
}

func isVersionZeroDotFour(d []byte) bool {
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
