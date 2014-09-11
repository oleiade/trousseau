package trousseau

import (
	"os"
)

// Global data store file path
var gStorePath string
func SetStorePath(storePath string) { gStorePath = storePath }
func GetStorePath() string          { return gStorePath }

// Gnupg trousseau master gpg key id
var gMasterGpgId string = os.Getenv(ENV_MASTER_GPG_ID_KEY)

// Keyring manager service and username to use in order to
// retrieve trousseau main gpg key passphrase from system
// keyring
var gKeyringService string = os.Getenv(ENV_KEYRING_SERVICE_KEY)
var gKeyringUser string = os.Getenv(ENV_KEYRING_USER_KEY)
