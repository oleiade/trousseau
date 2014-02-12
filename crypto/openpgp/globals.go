package gpg

import (
    "os"
    "path/filepath"
)

// Gnupg keyrings files
var gPubringFile string = filepath.Join(os.Getenv("HOME"), ".gnupg", "pubring.gpg")
var gSecringFile string = filepath.Join(os.Getenv("HOME"), ".gnupg", "secring.gpg")

// Gnupg trousseau master gpg key id
var gMasterGpgId string = os.Getenv(ENV_MASTER_GPG_ID_KEY)
