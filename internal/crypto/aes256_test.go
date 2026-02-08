package crypto

import (
	"crypto/aes"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"golang.org/x/crypto/scrypt"
)

type AES256ServiceTestSuite struct {
	suite.Suite

	Passphrase string
	Salt       []byte
	ScryptKey  []byte

	Cipher *AES256Cipher

	Input          string
	EncryptedInput string
}

func TestVaultTestSuite(t *testing.T) {
	suite.Run(t, new(AES256ServiceTestSuite))
}

func (suite *AES256ServiceTestSuite) SetupSuite() {
	suite.Passphrase = "passphrase"

	salt, err := GenerateSalt(SaltSize)
	if err != nil {
		suite.T().Fatal(err)
	}
	suite.Salt = salt

	scryptKey, err := scrypt.Key([]byte(suite.Passphrase), suite.Salt, KeyCost, aes.BlockSize, 1, 32)
	if err != nil {
		suite.T().Fatal(err)
	}
	suite.ScryptKey = scryptKey
	suite.Input = "input"
}

func (suite *AES256ServiceTestSuite) TearDownSuite() {
}

func (suite *AES256ServiceTestSuite) TestAES256Service_Encrypt() {
	cipher, err := NewAES256Cipher(suite.Passphrase, suite.Salt)
	if err != nil {
		suite.T().Errorf("NewAES256Cipher() err = %v\n", err)
	}

	service := &AES256Service{Cipher: cipher, Passphrase: suite.Passphrase}
	encrypted, err := service.Encrypt([]byte(suite.Input))
	if err != nil {
		suite.T().Errorf("AES256Cipher.Encrypt() err = %v\n", err)
	}

	assert.NotEmpty(suite.T(), encrypted)
	assert.True(suite.T(), len(encrypted) > SaltSize+aes.BlockSize)
}

func (suite *AES256ServiceTestSuite) TestAES256Service_Decrypt() {
	cipher, err := NewAES256Cipher(suite.Passphrase, suite.Salt)
	if err != nil {
		suite.T().Errorf("NewAES256Cipher() err = %v\n", err)
	}

	service := &AES256Service{Cipher: cipher, Passphrase: suite.Passphrase}
	encrypted, err := service.Encrypt([]byte(suite.Input))
	if err != nil {
		suite.T().Errorf("AES256Cipher.Encrypt() err = %v\n", err)
	}

	decrypted, err := service.Decrypt(encrypted)
	if err != nil {
		suite.T().Errorf("AES256Cipher.Decrypt() err = %v\n", err)
	}

	assert.Equal(suite.T(), suite.Input, string(decrypted))
}

func (suite *AES256ServiceTestSuite) TestNewAES256Cipher() {
	type args struct {
		passphrase string
		salt       []byte
	}
	tests := []struct {
		name    string
		args    args
		want    *AES256Cipher
		wantErr bool
	}{
		{
			name:    "it should produce an error when provided an invalid size salt",
			args:    args{passphrase: suite.Passphrase, salt: []byte("tooshort")},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should produce an error when provided a nil salt",
			args:    args{passphrase: suite.Passphrase, salt: nil},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should generate a valid Key when provided a valid passphrase and salt",
			args:    args{passphrase: suite.Passphrase, salt: suite.Salt},
			want:    &AES256Cipher{Key: suite.ScryptKey, Salt: suite.Salt},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		suite.T().Run(tt.name, func(t *testing.T) {
			got, err := NewAES256Cipher(tt.args.passphrase, tt.args.salt)
			if (err != nil) != tt.wantErr {
				t.Errorf("NewAES256Cipher() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(tt.want, got, cmpopts.IgnoreFields(AES256Cipher{}, "Block")); diff != "" {
				t.Errorf("NewAES256Cipher() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func (suite *AES256ServiceTestSuite) Test_GenerateSalt() {
	tests := []struct {
		name     string
		wantSize int
		wantErr  bool
	}{
		{
			name:     "it should generate a salt with a valid size",
			wantSize: SaltSize,
			wantErr:  false,
		},
	}
	for _, tt := range tests {
		suite.T().Run(tt.name, func(t *testing.T) {
			got, err := GenerateSalt(SaltSize)
			if (err != nil) != tt.wantErr {
				t.Errorf("GenerateSalt() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(tt.wantSize, len(got)); diff != "" {
				t.Errorf("GenerateSalt() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
