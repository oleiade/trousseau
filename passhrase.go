package trousseau

import (
	"github.com/tmc/keyring"
	"os"
)

// GetPassphrase attemps to retrieve the user's gpg master
// key passphrase using multiple methods. First it will attempt
// to retrieve it from the environment, then it will try to fetch
// it from the system keyring manager, ultimately it will try
// to get it from a running gpg-agent daemon.
func GetPassphrase() string {
	var passphrase string

	// Try to retrieve passphrase from env
	passphrase = os.Getenv(ENV_PASSPHRASE_KEY)
	if len(passphrase) > 0 {
		return passphrase
	}

	// If passphrase wasn't found in env, try to fetch it from
	// system keyring manager.
	passphrase, _ = keyring.Get(gKeyringService, gKeyringUser)

	return passphrase

}
