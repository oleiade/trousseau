// Package tempura provides temporary files creation and manipulation helpers
// for the purposes of enhancing tests involving files creation.
package tempura

import (
	"io/ioutil"
	"os"
)

// TempFile represents an open temporary file descriptor
type TempFile struct {
	*os.File

	dir    string
	prefix string
}

// FromBytes creates a new temporary file in the directory dir with a name beginning with prefix,
// opens the file for reading and writing, writes the provided data into it
// and returns seeks the underlying file object to 0.
//
// If dir is the empty string, TempFile uses the default directory for temporary files (see os.TempDir).
// Multiple programs calling TempFile simultaneously will not choose the same file.
// The caller can use f.Name() to find the pathname of the file.
// It is the caller's responsibility to remove the file when no longer needed.
func FromBytes(dir, prefix string, data []byte) (*TempFile, error) {
	var tmp *TempFile = &TempFile{dir: dir, prefix: prefix}
	var err error

	tmp.File, err = ioutil.TempFile(dir, prefix)
	if err != nil {
		return nil, err
	}

	_, err = tmp.Write(data)
	if err != nil {
		return nil, err
	}

	tmp.Seek(0, 0)

	return tmp, nil
}

// Create a new temporary file in the directory dir with a name beginning with prefix,
// writes the provided data into it and returns it's path.
//
// If dir is the empty string, TempFile uses the default directory for temporary files (see os.TempDir).
// Multiple programs calling TempFile simultaneously will not choose the same file.
// It is the caller's responsibility to remove the file when no longer needed.
func Create(dir, prefix string, data []byte) (path string, err error) {
	var tmp *os.File

	tmp, err = ioutil.TempFile(dir, prefix)
	if err != nil {
		return "", err
	}
	defer tmp.Close()

	_, err = tmp.Write(data)
	if err != nil {
		return "", err
	}

	return tmp.Name(), nil
}
