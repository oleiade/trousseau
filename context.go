package trousseau

import (
	"os"
	"path/filepath"
	"github.com/tmc/keyring"
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

// Global data store file path
var gStorePath string
func SetStorePath(storePath string) { gStorePath = storePath }
func GetStorePath() string          { return gStorePath }

func InferStorePath() string {
	envPath := os.Getenv(ENV_TROUSSEAU_STORE)
	contextPath := GetStorePath()

	if envPath != "" {
		return envPath
	} else if contextPath != "" {
		return contextPath
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

	// try to retrieve passphrase from env
	passphrase = os.Getenv(ENV_PASSPHRASE_KEY)
	if len(passphrase) > 0 {
		return passphrase
	}

	// if passphrase wasn't found in env, try to fetch it from
	// system keyring manager.
	passphrase, err = keyring.Get(os.Getenv(ENV_KEYRING_SERVICE_KEY), os.Getenv(ENV_KEYRING_USER_KEY))
	if len(passphrase) > 0 {
		return passphrase
	}

	// if passphrase was enither found in the environment nor
	// system keyring manager try to fetch it from gpg-agent
	if os.Getenv("gpg_agent_info") != "" {
		passphrase, err = getGpgPassphrase(os.Getenv(ENV_MASTER_GPG_ID_KEY))
	}

	if err != nil {
		ErrorLogger.Fatal("no passphrase provided. unable to open data store")
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
