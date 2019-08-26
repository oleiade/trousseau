package trousseau

import (
	"errors"
	"os"
	"path/filepath"

	"github.com/oleiade/trousseau/internal/config"
	"github.com/oleiade/trousseau/pkg/gpgagent"
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
var gPassphrase string
var gPassphraseAskFlag bool

func SetStorePath(storePath string) { gStorePath = storePath }
func GetStorePath() string          { return gStorePath }
func SetPassphrase(p string)        { gPassphrase = p }

func InferStorePath(c *config.Config) string {
	contextPath := GetStorePath()

	if contextPath != "" {
		return contextPath
	} else if c.StorePath != "" {
		return c.StorePath
	}

	return filepath.Join(os.Getenv("HOME"), c.Filename)
}

func AskPassphraseFlagCheck() bool {
	return gPassphraseAskFlag
}

func SetAsked() {
	gPassphraseAskFlag = true
}

func AskPassphrase(confirm bool) {
	if confirm {
		SetPassphrase(PromptForHiddenInputConfirm())
	} else {
		SetPassphrase(PromptForHiddenInput("Passphrase: "))
	}

	// Set the global AskPassphraseFlag so as to not ask again
	SetAsked()
}

// GetPassphrase attemps to retrieve the user's gpg master
// key passphrase using multiple methods. First it will attempt
// to retrieve it from the environment, then it will try to fetch
// it from the system keyring manager, ultimately it will try
// to get it from a running gpg-agent daemon.
func GetPassphrase(c *config.Config) (passphrase string, err error) {
	//var err error

	if gPassphrase != "" {
		return gPassphrase, nil
	}

	// try to retrieve passphrase from env
	if len(c.Passphrase) > 0 {
		return passphrase, nil
	}

	// if passphrase wasn't found in env, try to fetch it from
	// system keyring manager.
	passphrase, err = keyring.Get(c.Keyring.ServiceKey, c.Keyring.UserKey)
	if len(passphrase) > 0 {
		return passphrase, nil
	}

	// if passphrase was enither found in the environment nor
	// system keyring manager try to fetch it from gpg-agent
	if os.Getenv("GPG_AGENT_INFO") != "" {
		passphrase, err = getGpgPassphrase(c)
	}

	if err != nil {
		// ErrorLogger.Fatal("no passphrase provided. unable to open data store")
		return "", errors.New("no passphrase provided. unable to open data store")
	}

	return passphrase, nil
}

func getGpgPassphrase(c *config.Config) (string, error) {
	conn, err := gpgagent.NewGpgAgentConn()
	if err != nil {
		return "", err
	}

	passphraseRequest := &gpgagent.PassphraseRequest{CacheKey: c.MasterGPGID}
	passphrase, err := conn.GetPassphrase(passphraseRequest)
	if err != nil {
		return "", err
	}

	return passphrase, nil
}
