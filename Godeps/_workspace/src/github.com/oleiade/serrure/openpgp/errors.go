package openpgp

type ErrorCode int

// OpenPGP error types constants
const (
	ERR_KEYRING = iota
)

// Encryption error types constants
const (
	ERR_ENCRYPTION_ENCODING ErrorCode = iota
	ERR_ENCRYPTION_ENCRYPT
)

// Decryption error types constants
const (
	ERR_DECRYPTION_KEYS ErrorCode = iota
	ERR_DECRYPTION_HASHES
	ERR_DECRYPTION_CIPHERS
)

type OpenPGPError struct {
	Code ErrorCode
	msg  string
}

func (e *OpenPGPError) Error() string {
	return e.msg
}

func NewOpenPGPError(code ErrorCode, msg string) *OpenPGPError {
	return &OpenPGPError{
		Code: code,
		msg:  msg,
	}
}
