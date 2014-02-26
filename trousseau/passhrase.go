package trousseau

import (
	"github.com/tmc/keyring"
	"log"
	"os"
)

// GetPassphrase attemps to retrieve the user's gpg master
// key passphrase using multiple methods. First it will attempt
// to retrieve it from the environment, then it will try to fetch
// it from the system keyring manager, ultimately it will try
// to get it from a running gpg-agent daemon.
func GetPassphrase() (passphrase string) {
	var err error

	// Try to retrieve passphrase from env
	passphrase = os.Getenv(ENV_PASSPHRASE_KEY)
	if len(passphrase) > 0 {
		return passphrase
	}

	// If passphrase wasn't found in env, try to fetch it from
	// system keyring manager.
	passphrase, err = keyring.Get(gKeyringService, gKeyringUser)
	if len(passphrase) > 0 {
		return passphrase
	}

	// If passphrase was enither found in the environment nor
	// system keyring manager try to fetch it from gpg-agent
	if os.Getenv("GPG_AGENT_INFO") != "" {
		passphrase, err = GetGpgPassphrase(gMasterGpgId)
	}

	if err != nil {
		log.Fatal("No passphrase provided. Unable to open data store")
	}

	return passphrase
}

func GetGpgPassphrase(gpgId string) (string, error) {
	conn, err := NewGpgAgentConn()
	if err != nil {
		return "", err
	}

	passphraseRequest := &PassphraseRequest{CacheKey: gpgId}
	passphrase, err := conn.GetPassphrase(passphraseRequest)
	if err != nil {
		return "", err
	}

	return passphrase, nil
}
