package trousseau

import (
	"os"
	"path"
)

// Declare encryption types
type CryptoType int

const (
	SYMMETRIC_ENCRYPTION  CryptoType = 0
	ASYMMETRIC_ENCRYPTION CryptoType = 1

	SYMMETRIC_ENCRYPTION_REPR  string = "symmetric"
	ASYMMETRIC_ENCRYPTION_REPR string = "asymmetric"
)

// Declare available encryption algorithms
type CryptoAlgorithm int

const (
	GPG_ENCRYPTION     CryptoAlgorithm = 0
	AES_256_ENCRYPTION CryptoAlgorithm = 1

	GPG_ENCRYPTION_REPR     string = "gpg"
	AES_256_ENCRYPTION_REPR string = "aes256"
)

// Gnupg variables
var GnupgHome = path.Join(os.Getenv("HOME"), ".gnupg")
var GnupgPubring func() string = func() string { return path.Join(GnupgHome, "pubring.gpg") }
var GnupgSecring func() string = func() string { return path.Join(GnupgHome, "secring.gpg") }
