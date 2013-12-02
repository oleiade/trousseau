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

// Gnupg trousseau master gpg key id
var gMasterGpgId string = os.Getenv(ENV_MASTER_GPG_ID_KEY)

// Ssh default identity file path
var gPrivateRsaKeyPath string = filepath.Join(os.Getenv("HOME"), ".ssh", "id_rsa")

// Keyring manager service and username to use in order to
// retrieve trousseau main gpg key passphrase from system
// keyring
var gKeyringService string = os.Getenv(ENV_KEYRING_SERVICE_KEY)
var gKeyringUser string = os.Getenv(ENV_KEYRING_USER_KEY)

// S3 and Scp dsn default values
var gS3Defaults map[string]string = map[string]string{
	"Path": "trousseau.tsk",
}
var gScpDefaults map[string]string = map[string]string{
	"Id":   os.Getenv("USER"),
	"Port": "22",
	"Path": "trousseau.tsk",
}
