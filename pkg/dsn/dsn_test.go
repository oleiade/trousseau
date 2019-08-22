package dsn

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestParse_valid_s3(t *testing.T) {
	rawdsn := "s3://1238919ahfnbkd8:wediomeof7wef98+qjefoiqed/niqwef@mybucket:eu-west-1/testpath"
	dsn, err := Parse(rawdsn)
	assert.NoError(t, err)

	assert.Equal(t, dsn.Scheme, "s3")
	assert.Equal(t, dsn.Id, "1238919ahfnbkd8")
	assert.Equal(t, dsn.Secret, "wediomeof7wef98+qjefoiqed/niqwef")
	assert.Equal(t, dsn.Host, "mybucket")
	assert.Equal(t, dsn.Port, "eu-west-1")
	assert.Equal(t, dsn.Path, "testpath")
}

// func TestParse_invalid_s3(t *testing.T) {
//     invalidRegionDsn := "s3://1238919ahfnbkd8:wediomeof7wef98+qjefoiqed/niqwef@mybucket:bla/testpath"
//     // invalidSecretDsn := "s3://1238919ahfnbkd8:in@mybucket:bla/testpath"
//
//     _, err := Parse(invalidRegionDsn)
//     assert.Error(t, err)
// }

func TestSetDefaults_apply_s3_defaults_to_empty_dsn(t *testing.T) {
	dsn := new(Dsn)

	dsn.SetDefaults(map[string]string{"Path": "trousseau.tsk"})
	assert.Equal(t, dsn.Path, "trousseau.tsk")
}

func TestSetDefaults_apply_s3_defaults_to_nonempty_dsn(t *testing.T) {
	dsn := &Dsn{Path: "abc 123"}

	dsn.SetDefaults(map[string]string{"Path": "easy as do re mi"})
	assert.Equal(t, dsn.Path, "abc 123")
}

func TestSetDefaults_apply_scp_defaults_to_empty_dsn(t *testing.T) {
	dsn := new(Dsn)

	err := dsn.SetDefaults(map[string]string{"Port": "22", "Path": "trousseau.tsk"})
	assert.NoError(t, err)
	assert.Equal(t, dsn.Port, "22")
	assert.Equal(t, dsn.Path, "trousseau.tsk")
}

func TestSetDefaults_apply_scp_defaults_to_nonempty(t *testing.T) {
	dsn := &Dsn{Port: "2121"}

	err := dsn.SetDefaults(map[string]string{"Port": "22"})
	assert.NoError(t, err)
	assert.Equal(t, dsn.Port, "2121")
}

func TestSetDeafaults_with_invalid_key(t *testing.T) {
	dsn := new(Dsn)

	err := dsn.SetDefaults(map[string]string{"Fail,bitch": "Okay master"})
	assert.Error(t, err)
}
