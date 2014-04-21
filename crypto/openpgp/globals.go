package openpgp

import (
	"os"
	"path/filepath"
)

// Gnupg keyrings files
var gPubringFile string = func() string {
	envPubring := os.Getenv("GNUPG_PUBRING_PATH")

	if envPubring != "" {
		return envPubring
	}

	return filepath.Join(os.Getenv("HOME"), ".gnupg", "pubring.gpg")
}()

var gSecringFile string = func() string {
	envSecring := os.Getenv("GNUPG_SECRING_PATH")

	if envSecring != "" {
		return envSecring
	}

	return filepath.Join(os.Getenv("HOME"), ".gnupg", "secring.gpg")
}()

// Gnupg trousseau master gpg key id
var gMasterGpgId string = os.Getenv(ENV_MASTER_GPG_ID_KEY)
