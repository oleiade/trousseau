package trousseau

import (
	"bytes"
	"io"
	"io/ioutil"
	"log"
	"os"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/oleiade/serrure/openpgp"
	"github.com/oleiade/trousseau/internal/config"
	"github.com/stretchr/testify/suite"
)

type VaultTestSuite struct {
	suite.Suite

	InvalidContentTypeFilePath string
	V030FilePath               string
	VLatestFilePath            string
}

func TestVaultTestSuite(t *testing.T) {
	suite.Run(t, new(VaultTestSuite))
}

func (suite *VaultTestSuite) SetupSuite() {
	crateTemporaryFile := func(d []byte) (string, error) {
		// Create a temporary file with invalid content type (unparasable)
		f, err := ioutil.TempFile("", "trousseau")
		if err != nil {
			return "", err
		}

		_, err = f.Write(d)
		if err != nil {
			return "", err
		}

		return f.Name(), nil
	}

	p, err := crateTemporaryFile([]byte("thisisgibberish"))
	if err != nil {
		log.Fatal(err)
	}
	suite.InvalidContentTypeFilePath = p

	p, err = crateTemporaryFile([]byte(openpgp.PGP_MESSAGE_HEADER))
	if err != nil {
		log.Fatal(err)
	}
	suite.V030FilePath = p

	p, err = crateTemporaryFile([]byte(`{"crypto_type": 0, "crypto_algorithm": 0, "_data": ""}`))
	if err != nil {
		log.Fatal(err)
	}
	suite.VLatestFilePath = p
}

func (suite *VaultTestSuite) TearDownSuite() {
	err := os.Remove(suite.InvalidContentTypeFilePath)
	if err != nil {
		log.Fatal(err)
	}

	err = os.Remove(suite.V030FilePath)
	if err != nil {
		log.Fatal(err)
	}

	err = os.Remove(suite.VLatestFilePath)
	if err != nil {
		log.Fatal(err)
	}
}

func (suite *VaultTestSuite) TestOpenVault() {
	type args struct {
		fp string
	}
	tests := []struct {
		name    string
		args    args
		want    *Vault
		wantErr bool
	}{
		{
			name:    "it should return an error if the file does not exist",
			args:    args{fp: "/does/not/exist"},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should return an error if the file does not contain valid JSON",
			args:    args{fp: suite.InvalidContentTypeFilePath},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should return an error if the file contains the v0.3.0 file format",
			args:    args{fp: suite.V030FilePath},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should return a valid Vault on success",
			args:    args{fp: suite.VLatestFilePath},
			want:    &Vault{CryptoAlgorithm: GPG_ENCRYPTION, CryptoType: SYMMETRIC_ENCRYPTION, Encrypted: []byte{}},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		suite.T().Run(tt.name, func(t *testing.T) {
			got, err := OpenVault(tt.args.fp)
			if (err != nil) != tt.wantErr {
				t.Errorf("OpenVault() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(tt.want, got, cmpopts.IgnoreUnexported(Vault{})); diff != "" {
				t.Errorf("OpenVault() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func (suite *VaultTestSuite) TestVault_Unlock() {
	type fields struct {
		CryptoType      CryptoType
		CryptoAlgorithm CryptoAlgorithm
		Encrypted       []byte
		cryptoMapping   map[CryptoAlgorithm]interface{}
	}
	type args struct {
		c *config.Config
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    *SecretStore
		wantErr bool
	}{}
	for _, tt := range tests {
		suite.T().Run(tt.name, func(t *testing.T) {
			v := &Vault{
				CryptoType:      tt.fields.CryptoType,
				CryptoAlgorithm: tt.fields.CryptoAlgorithm,
				Encrypted:       tt.fields.Encrypted,
				cryptoMapping:   tt.fields.cryptoMapping,
			}
			got, err := v.Unlock(tt.args.c)
			if (err != nil) != tt.wantErr {
				t.Errorf("Vault.Unlock() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(tt.want, got, cmpopts.IgnoreUnexported(Vault{})); diff != "" {
				t.Errorf("Vault.Unlock() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func (suite *VaultTestSuite) TestVault_Lock() {
	type fields struct {
		CryptoType      CryptoType
		CryptoAlgorithm CryptoAlgorithm
		Encrypted       []byte
		cryptoMapping   map[CryptoAlgorithm]interface{}
	}
	type args struct {
		c  *config.Config
		ss *SecretStore
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		wantErr bool
	}{
		// TODO: Add test cases.
	}
	for _, tt := range tests {
		suite.T().Run(tt.name, func(t *testing.T) {
			v := &Vault{
				CryptoType:      tt.fields.CryptoType,
				CryptoAlgorithm: tt.fields.CryptoAlgorithm,
				Encrypted:       tt.fields.Encrypted,
				cryptoMapping:   tt.fields.cryptoMapping,
			}
			if err := v.Lock(tt.args.c, tt.args.ss); (err != nil) != tt.wantErr {
				t.Errorf("Vault.Lock() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func (suite *VaultTestSuite) TestReadVault() {
	type args struct {
		r io.Reader
	}
	tests := []struct {
		name    string
		args    args
		want    *Vault
		wantErr bool
	}{
		{
			name:    "it should return the default Vault if the Reader reads 0",
			args:    args{r: bytes.NewBuffer([]byte{})},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should return an error if the Reader does not read valid JSON",
			args:    args{r: bytes.NewBuffer([]byte(`thisisgibberish`))},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should return an error if the Reader reads content from the v0.3.0 file format",
			args:    args{r: bytes.NewBuffer([]byte(openpgp.PGP_MESSAGE_HEADER))},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should return a valid Vault on success",
			args:    args{r: bytes.NewReader([]byte(`{"crypto_type": 0, "crypto_algorithm": 0, "_data": ""}`))},
			want:    &Vault{CryptoAlgorithm: GPG_ENCRYPTION, CryptoType: SYMMETRIC_ENCRYPTION, Encrypted: []byte{}},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		suite.T().Run(tt.name, func(t *testing.T) {
			got, err := ReadVault(tt.args.r)
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadVault() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(tt.want, got, cmpopts.IgnoreUnexported(Vault{})); diff != "" {
				t.Errorf("ReadVault() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
