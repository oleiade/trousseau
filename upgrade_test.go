package trousseau

import (
	"encoding/json"
	"github.com/oleiade/serrure/openpgp"
	"testing"
)

func TestIsVersionZeroDotThree(t *testing.T) {
	data := []byte(openpgp.PGP_MESSAGE_HEADER +
		"12kjd091jd192jd0192jd" +
		openpgp.PGP_MESSAGE_FOOTER)

	assert(t,
		isVersionZeroDotThreeDotZero(data) == true,
		"Input test data were suppose to match version 0.3.N")
}

func TestIsVersionZeroDotThree_fails_with_data_shorter_than_pgp_header(t *testing.T) {
	data := []byte("abc123")
	assert(t,
		isVersionZeroDotThreeDotZero(data) == false,
		"Input test data weren't suppose to match version 0.3.N")
}

func TestIsVersionZeroDotFour(t *testing.T) {
	store := map[string]interface{}{
		"crypto_algorithm": 0,
		"crypto_type":      1,
		"_data":            "oqwimdoqiwmd0qwd0iq0wdijqw9d0",
	}

	data, err := json.Marshal(store)
	if err != nil {
		t.Error(err)
	}

	assert(t,
		isVersionZeroDotThreeDotOne(data) == true,
		"Input test data were suppose to match version 0.4.N")
}

func TestDiscoverVersion_with_only_one_valid_version_in_mapping(t *testing.T) {
	var data []byte = []byte(openpgp.PGP_MESSAGE_HEADER +
		"12kjd091jd192jd0192jd" +
		openpgp.PGP_MESSAGE_FOOTER)
	var mapping map[string]VersionMatcher = map[string]VersionMatcher{
		"0.3.0": isVersionZeroDotThreeDotZero,
	}

	assert(t,
		DiscoverVersion(data, mapping) == "0.3.0",
		"Version 0.3.0 was supposed to be discovered")
}

func TestDiscoverVersion_with_two_valid_versions_in_mapping(t *testing.T) {
	var store map[string]interface{} = map[string]interface{}{
		"crypto_algorithm": 0,
		"crypto_type":      1,
		"_data":            "oqwimdoqiwmd0qwd0iq0wdijqw9d0",
	}
	var mapping map[string]VersionMatcher = map[string]VersionMatcher{
		"0.3.0": isVersionZeroDotThreeDotZero,
		"0.4.0": isVersionZeroDotThreeDotOne,
	}

	data, err := json.Marshal(store)
	if err != nil {
		t.Error(err)
	}

	assert(t,
		DiscoverVersion(data, mapping) == "0.4.0",
		"Version 0.4.0 was supposed to be discovered")
}

func TestDiscoverVersion_with_two_matching_version_returns_the_lowest(t *testing.T) {
	var data []byte = []byte(openpgp.PGP_MESSAGE_HEADER +
		"12kjd091jd192jd0192jd" +
		openpgp.PGP_MESSAGE_FOOTER)
	var mapping map[string]VersionMatcher = map[string]VersionMatcher{
		"0.3.0": isVersionZeroDotThreeDotZero,
		"0.3.1": isVersionZeroDotThreeDotZero,
	}

	assert(t,
		DiscoverVersion(data, mapping) == "0.3.0",
		"Version 0.3.0 was supposed to be discovered")

}

func TestDiscoverVersion_with_no_matching_version(t *testing.T) {
	var data []byte = []byte("abc")
	var mapping map[string]VersionMatcher = map[string]VersionMatcher{
		"0.3.0": isVersionZeroDotThreeDotZero,
		"0.4.0": isVersionZeroDotThreeDotOne,
	}

	equals(t, DiscoverVersion(data, mapping), "")
}
