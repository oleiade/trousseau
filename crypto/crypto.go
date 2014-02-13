package crypto

type Algorithm int

const (
	GPG_ENCRYPTION Algorithm = iota
	AES_ENCRYPTION
)

type Options struct {
	Algorithm  Algorithm
	Passphrase string
	Recipients []string
}

func NewOptions(alg Algorithm, passphrase string) *Options {
	return &Options{
		Algorithm:  alg,
		Passphrase: passphrase,
	}
}
