package crypto

import "io"

type Decrypter interface {
	Decrypt(r io.Reader, w io.Writer) error
}

type Encrypter interface {
	Encrypt(r io.Reader, w io.Writer) error
}

type CryptoService interface {
	Decrypter
	Encrypter
}
