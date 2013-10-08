package trousseau

import (
	"bytes"
	"code.google.com/p/go.crypto/ssh"
	"crypto"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"io"
	"io/ioutil"
	"strings"
)

type ScpStorage struct {
	host      string
	port      string
	connexion *ssh.ClientConn

	Keychain *Keychain
	User     string
	Endpoint string
}

type Keychain struct {
	key *rsa.PrivateKey
}

func NewScpStorage(host, port, user string, keychain *Keychain) *ScpStorage {
	return &ScpStorage{
		Keychain: keychain,
		User:     user,
		Endpoint: strings.Join([]string{host, port}, ":"),
		host:     host,
		port:     port,
	}
}

func NewKeychain(key *rsa.PrivateKey) *Keychain {
	return &Keychain{
		key: key,
	}
}

func DecodePrivateKeyFromFile(privateKeyPath string) (*rsa.PrivateKey, error) {
	keyContent, err := ioutil.ReadFile(privateKeyPath)
	if err != nil {
		return nil, err
	}

	block, _ := pem.Decode([]byte(keyContent))
	rsakey, err := x509.ParsePKCS1PrivateKey(block.Bytes)
	if err != nil {
		return nil, err
	}

	return rsakey, nil
}

func (k *Keychain) Key(i int) (key ssh.PublicKey, err error) {
	if i != 0 {
		return nil, nil
	}
	
    // Transform the rsa key into an ssh key
    ssh_publickey, _ := ssh.NewPublicKey(k.key.PublicKey)

	return ssh_publickey, nil
}

func (k *Keychain) Sign(i int, rand io.Reader, data []byte) (sig []byte, err error) {
	hashFunc := crypto.SHA1
	h := hashFunc.New()
	h.Write(data)
	digest := h.Sum(nil)
	return rsa.SignPKCS1v15(rand, k.key, hashFunc, digest)
}

func (ss *ScpStorage) Connect() error {
	var err error

	clientConfig := &ssh.ClientConfig{
		User: ss.User,
		Auth: []ssh.ClientAuth{
			ssh.ClientAuthKeyring(ss.Keychain),
		},
	}

	ss.connexion, err = ssh.Dial("tcp", ss.Endpoint, clientConfig)
	if err != nil {
		return fmt.Errorf("Failed to dial: %s", err.Error())
	}

	return nil
}

func (ss *ScpStorage) Push(remoteName string) error {
	session, err := ss.connexion.NewSession()
	if err != nil {
		return fmt.Errorf("Failed to create session: %s", err.Error())
	}
	defer session.Close()

	go func() {
		w, _ := session.StdinPipe()
		defer w.Close()

		content, _ := ioutil.ReadFile(gStorePath)

		fmt.Fprintln(w, "C0755", len(content), remoteName)
		fmt.Fprint(w, string(content))
		fmt.Fprint(w, "\x00")
	}()

	if err := session.Run("/usr/bin/scp -qrt ./"); err != nil {
		return fmt.Errorf("Failed to run: %s", err.Error())
	}

	return nil
}

func (ss *ScpStorage) Pull(remoteName string) error {
	session, err := ss.connexion.NewSession()
	if err != nil {
		return fmt.Errorf("Failed to create session: %s", err.Error())
	}
	defer session.Close()

	var remoteFileBuffer bytes.Buffer
	session.Stdout = &remoteFileBuffer

	if err := session.Run(fmt.Sprintf("cat %s", remoteName)); err != nil {
		return fmt.Errorf("Failed to run: %s", err.Error())
	}

	err = ioutil.WriteFile(gStorePath, remoteFileBuffer.Bytes(), 0744)
	if err != nil {
		return err
	}

	return nil
}
