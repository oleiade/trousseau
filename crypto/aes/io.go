package aes

import (
	"io/ioutil"
	"os"
)

type AESFile struct {
	file       *os.File
	key        AES256Key
	passphrase string
	Path       string
}

func NewAES256File(filepath string, key AES256Key) *AESFile {
	return &AESFile{
		Path: filepath,
		key:  key,
	}
}

func OpenFile(name string, mode int, key AES256Key) (*AESFile, error) {
	f, err := os.OpenFile(name, mode, os.FileMode(0600))
	if err != nil {
		return nil, err
	}
	aesfile := NewAES256File(name, key)
	aesfile.file = f

	return aesfile, nil
}

func (aesf *AESFile) Close() error {
	return aesf.file.Close()
}

func (aesf *AESFile) ReadAll() ([]byte, error) {
	encryptedData, err := ioutil.ReadAll(aesf.file)
	if err != nil {
		return nil, err
	}
	plainData, err := Decrypt(aesf.passphrase, encryptedData)
	if err != nil {
		return nil, err
	}
	return plainData, nil
}

func (aesf *AESFile) Write(d []byte) (n int, err error) {
	encData, err := Encrypt(aesf.passphrase, d)
	if err != nil {
		return 0, err
	}

	err = aesf.file.Truncate(int64(len(encData)))
	n, err = aesf.file.Write(encData)
	return n, err
}
