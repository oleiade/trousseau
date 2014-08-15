package trousseau

const TROUSSEAU_VERSION = "0.3.0"

const (
	DEFAULT_STORE_FILENAME = ".trousseau"
)

const (
	CONFIG_KEY_RECIPIENTS = "recipients"
	CONFIG_KEY_PASSWORD   = "password"
)

const (
	ENV_TROUSSEAU_STORE     = "TROUSSEAU_STORE"
	ENV_PASSPHRASE_KEY      = "TROUSSEAU_PASSPHRASE"
	ENV_KEYRING_SERVICE_KEY = "TROUSSEAU_KEYRING_SERVICE"
	ENV_KEYRING_USER_KEY    = "USER"
	ENV_MASTER_GPG_ID_KEY   = "TROUSSEAU_MASTER_GPG_ID"
	ENV_SSH_PRIVATE_KEY     = "TROUSSEAU_PRIVATE_KEY"
)

// Declare encryption types
type EncryptionType int
const (
	SYMMETRIC_ENCRYPTION EncryptionType = iota
	ASYMMETRIC_ENCRYPTION EncryptionType
)

// Declare available encryption algorithms
type EncryptionAlgorithm int
const (
	GPG_ENCRYPTION EncryptionAlgorithm = iota
	AES_256_ENCRYPTION
)

// Import strategies enumeration
const (
	IMPORT_YOURS     = 0x0
	IMPORT_THEIRS    = 0x1
	IMPORT_OVERWRITE = 0x2
)
