package trousseau

import (
	"github.com/tmc/keyring"
	"log"
	"os"
	"path/filepath"
)

// Global variables defining default values for S3 and scp
// uploads/downloads
var (
	S3Defaults map[string]string = map[string]string{
		"Path": "trousseau.tsk",
	}
	ScpDefaults map[string]string = map[string]string{
		"Id":   os.Getenv("USER"),
		"Port": "22",
		"Path": "trousseau.tsk",
	}
)

func GetStorePath() string {
	envPath := os.Getenv(ENV_TROUSSEAU_STORE)

	if envPath != "" {
		return envPath
	}

	return filepath.Join(os.Getenv("HOME"), DEFAULT_STORE_FILENAME)
}

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
		passphrase, err = getGpgPassphrase(gMasterGpgId)
	}

	if err != nil {
		log.Fatal("No passphrase provided. Unable to open data store")
	}

	return passphrase
}

func getGpgPassphrase(gpgId string) (string, error) {
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
