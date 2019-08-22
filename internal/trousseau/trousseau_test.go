package trousseau

import (
	"encoding/json"
	"testing"

	"os"

	"github.com/oleiade/tempura"
	"github.com/stretchr/testify/assert"
)

func TestOpenTrousseau(t *testing.T) {
	testData := make(map[string]interface{})
	testData["crypto_type"] = ASYMMETRIC_ENCRYPTION
	testData["crypto_algorithm"] = GPG_ENCRYPTION
	testData["_data"] = []byte("abc")

	jsonData, _ := json.Marshal(&testData)
	tmp, _ := tempura.FromBytes("/tmp", "trousseau", jsonData)
	defer tmp.File.Close()
	defer os.Remove(tmp.File.Name())

	tr, err := OpenTrousseau(tmp.File.Name())
	if err != nil {
		t.Fatal(err)
	}

	assert.True(t, tr.CryptoType == ASYMMETRIC_ENCRYPTION, "Wrong encryption type")
	assert.True(t, tr.CryptoAlgorithm == GPG_ENCRYPTION, "Wrong encryption algorithm")
	assert.Equal(t, tr.Data, []byte("abc"))
}

func TestOpenTrousseau_returns_err_when_file_does_not_exist(t *testing.T) {
	_, err := OpenTrousseau("/does/not/exist")
	if err == nil {
		t.Fatalf("OpenTrousseau function didn't failed while loading non existing file")
	}
}
