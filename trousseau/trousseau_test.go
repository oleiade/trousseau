package trousseau

import (
	"testing"
	"encoding/json"

	"github.com/oleiade/tempura"
)

func TestOpenTrousseau(t *testing.T) {
	testData := make(map[string]interface{})
	testData["encryption_type"] = ASYMMETRIC_ENCRYPTION
	testData["encryption_algorithm"] = GPG_ENCRYPTION
	testData["_data"] = "qeiun91inwd918hnd913dn19dni1dn9183nd9138d"

	jsonData, _ := json.Marshal(&testData)
	tmp, _ := tempura.FromBytes("/tmp", "trousseau", jsonData)

	tr, err := OpenTrousseau(tmp.File.Name())
	if err != nil {
		t.Fatal(err)
	}

	assert(t, tr.EncryptionType == ASYMMETRIC_ENCRYPTION, "Wrong encryption type")
	assert(t, tr.EncryptionAlgorithm == GPG_ENCRYPTION, "Wrong encryption algorithm")
	assert(t, tr.Store == "qeiun91inwd918hnd913dn19dni1dn9183nd9138d", "Invalid data retrieved")
}

func TestOpenTrousseau_returns_err_when_file_does_not_exist(t *testing.T) {
	_, err := OpenTrousseau("/does/not/exist")
	if err == nil {
		t.Fatalf("OpenTrousseau function didn't failed while loading non existing file")
	}
}
