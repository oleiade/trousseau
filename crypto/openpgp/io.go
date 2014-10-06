package openpgp

import (
	"io"
	"io/ioutil"
	"os"

	_ "github.com/oleiade/trousseau/crypto"
)

type GpgFile struct {
	file       fileLike
	passphrase string

	Path       string
	Recipients []string
}

type fileLike interface {
	io.Reader
	io.Writer
	io.Closer
	Name() string
	Stat() (os.FileInfo, error)
	Truncate(int64) error
}

// NewGpgFile creates a new GPG file.
func NewGpgFile(filepath, passphrase string, recipients []string) *GpgFile {
	gf := &GpgFile{
		Path:       filepath,
		Recipients: recipients,
		passphrase: passphrase,
	}
	return gf
}

// WrapFile wraps a *os.File-like object into a *GpgFile.
func WrapFile(f fileLike, passphrase string, recipients []string) *GpgFile {
	return &GpgFile{
		Path:       f.Name(),
		passphrase: passphrase,
		Recipients: recipients,
		file:       f,
	}
}

// Open opens the named file for reading/writing, depending on the given mode.
// If successful, methods on the returned file can be used for reading or writing;
// the associated file descriptor has mode O_RDONLY.
// If there is an error, it will be of type *PathError.
//
// The passphrase is used only for decryption.
func OpenFile(name string, mode int, passphrase string, recipients []string) (*GpgFile, error) {
	if mode == 0 {
		mode = os.O_RDONLY
	}
	f, err := os.OpenFile(name, mode, os.FileMode(0600))
	if err != nil {
		return nil, err
	}

	return WrapFile(f, passphrase, recipients), nil
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

	return gf.file.Write(encData)
}

// Stat returns the FileInfo structure describing file.
// If there is an error, it will be of type *PathError.
func (gf *GpgFile) Stat() (fi os.FileInfo, err error) {
	return gf.file.Stat()
}

func (gf *GpgFile) Truncate(p int64) error {
	// As we were able to encrypt data, truncate source
	// file and write to it
	return gf.file.Truncate(p)
}
