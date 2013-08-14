package trousseau

import (
	"os"
	"path/filepath"
)

// Global data store file path
var gStorePath string = filepath.Join(os.Getenv("HOME"), STORE_FILENAME)

// Gnupg keyrings files
var gPubringFile string = filepath.Join(os.Getenv("HOME"), ".gnupg", "pubring.gpg")
var gSecringFile string = filepath.Join(os.Getenv("HOME"), ".gnupg", "secring.gpg")
