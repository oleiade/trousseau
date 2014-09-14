package openpgp

import (
	_ "github.com/oleiade/trousseau/crypto"
	"io/ioutil"
	"os"
)

type GpgFile struct {
	file       *os.File
	passphrase string

	Path       string
	Recipients []string
}

func NewGpgFile(filepath, passphrase string, recipients []string) *GpgFile {
	return &GpgFile{
		Path:       filepath,
		Recipients: recipients,
		passphrase: passphrase,
	}
}

// Open opens the named file for reading.
// If successful, methods on the returned file can be used for reading;
// the associated file descriptor has mode O_RDONLY.
// If there is an error, it will be of type *PathError.
func OpenFile(name string, mode int, passphrase string, recipients []string) (*GpgFile, error) {
	f, err := os.OpenFile(name, mode, os.FileMode(0600))
	if err != nil {
		return nil, err
	}

	gpg := NewGpgFile(name, passphrase, recipients)
	gpg.file = f

	return gpg, nil
}

// Close closes the GpgFile, rendering it unusable for I/O.
// It returns an error, if any.
func (gf *GpgFile) Close() error {
	return gf.file.Close()
}

func (gf *GpgFile) ReadAll() ([]byte, error) {
	encryptedData, err := ioutil.ReadAll(gf.file)
	if err != nil {
		return nil, err
	}

	// Decrypt store data
	decryptionKeys, err := ReadSecRing(SecringFile)
	if err != nil {
		return nil, err
	}

	plainData, err := Decrypt(encryptedData, decryptionKeys, gf.passphrase)
	if err != nil {
		return nil, err
	}

	return plainData, nil
}

// Read reads up to len(b) bytes from the GpgFile.
// It returns the number of bytes read and an error, if any.
// EOF is signaled by a zero count with err set to io.EOF.
func (gf *GpgFile) Read(b []byte) (n int, err error) {
	plainData, err := gf.ReadAll()
	if err != nil {
		return 0, err
	}

	// fulfill the provided byte slice with
	// the file data
	n = copy(b, plainData)

	return n, nil
}

// Write writes len(b) bytes to the GpgFile.
// It returns the number of bytes written and an error, if any.
// Write returns a non-nil error when n != len(b).
func (gf *GpgFile) Write(d []byte) (n int, err error) {
	encryptionKeys, err := ReadPubRing(PubringFile, gf.Recipients)
	if err != nil {
		return 0, err
	}

	encData, err := Encrypt(d, encryptionKeys)
	if err != nil {
		return 0, err
	}

	// As we were able to encrypt data, truncate source
	// file and write to it
	err = gf.file.Truncate(int64(len(encData)))
	n, err = gf.file.Write(encData)

	return n, err
}

// Stat returns the FileInfo structure describing file.
// If there is an error, it will be of type *PathError.
func (gf *GpgFile) Stat() (fi os.FileInfo, err error) {
	return gf.file.Stat()
}
