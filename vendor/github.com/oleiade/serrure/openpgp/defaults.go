package openpgp

import (
	"os"
	"path/filepath"
)

var DEFAULT_GNUPG_DIR = ".gnupg"
var DEFAULT_GNUPG_PUBRING_FILE = "pubring.gpg"
var DEFAULT_GNUPG_SECRING_FILE = "secring.gpg"
var DEFAULT_GNUPG_THRUSTDB_FILE = "thrustdb.gpg"

var DEFAULT_GNUPG_PUBRING = filepath.Join(
	os.Getenv("USER"),
	DEFAULT_GNUPG_DIR,
	DEFAULT_GNUPG_PUBRING_FILE,
)

var DEFAULT_GNUPG_SECRING = filepath.Join(
	os.Getenv("USER"),
	DEFAULT_GNUPG_DIR,
	DEFAULT_GNUPG_SECRING_FILE,
)
