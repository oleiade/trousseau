package ssh

import (
	"bytes"
	"code.google.com/p/gosshold/ssh"
	"crypto"
	"crypto/dsa"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
)

type Keychain struct {
	keys []interface{}
}

func (k *Keychain) AddPEMKey(privateKeyPath string) error {
	var rsakey interface{}
	var err error

	keyContent, err := ioutil.ReadFile(privateKeyPath)
	if err != nil {
		return err
	}

	block, _ := pem.Decode([]byte(keyContent))
	if block == nil {
		return errors.New("no block in key")
	}

	rsakey, err = x509.ParsePKCS1PrivateKey(block.Bytes)
	if err != nil {
		rsakey, err = x509.ParsePKCS8PrivateKey(block.Bytes)
	}

	if err != nil {
		return err
	}

	k.keys = append(k.keys, rsakey)

	return nil
}

func (k *Keychain) AddPEMKeyPassword(key string, password string) (err error) {
	block, _ := pem.Decode([]byte(key))
	bytes, _ := x509.DecryptPEMBlock(block, []byte(password))
	rsakey, err := x509.ParsePKCS1PrivateKey(bytes)
	if err != nil {
		return
	}

	k.keys = append(k.keys, rsakey)

	return
}

func (k *Keychain) Key(i int) (key ssh.PublicKey, err error) {
	if i < 0 || i >= len(k.keys) {
		return nil, nil
	}

	switch key := k.keys[i].(type) {
	case *rsa.PrivateKey:
		return ssh.NewPublicKey(&key.PublicKey)
	case *dsa.PrivateKey:
		return ssh.NewPublicKey(&key.PublicKey)
	}

	return nil, errors.New("ssh: Unknown key type")
}

func (k *Keychain) Sign(i int, rand io.Reader, data []byte) (sig []byte, err error) {
	hashFunc := crypto.SHA1
	h := hashFunc.New()
	h.Write(data)
	digest := h.Sum(nil)

	switch key := k.keys[i].(type) {
	case *rsa.PrivateKey:
		return rsa.SignPKCS1v15(rand, key, hashFunc, digest)
	}

	return nil, errors.New("ssh: Unknown key type")
}

func (ss *ScpStorage) Connect() error {
	var err error

	clientConfig := &ssh.ClientConfig{
		User: ss.User,
		Auth: []ssh.ClientAuth{
			ssh.ClientAuthPassword(password(ss.Password)),
			ssh.ClientAuthKeyring(ss.Keychain),
		},
	}

	ss.connexion, err = ssh.Dial("tcp", ss.Endpoint, clientConfig)
	if err != nil {
		return fmt.Errorf("Failed to dial: %s", err.Error())
	}

	return nil
}

func (ss *ScpStorage) Push(localPath, remotePath string) error {
	session, err := ss.connexion.NewSession()
	if err != nil {
		return fmt.Errorf("Failed to create session: %s", err.Error())
	}
	defer session.Close()

	go func() {
		w, _ := session.StdinPipe()
		defer w.Close()

		content, _ := ioutil.ReadFile(localPath)

		fmt.Fprintln(w, "C0755", len(content), remotePath)
		fmt.Fprint(w, string(content))
		fmt.Fprint(w, "\x00")
	}()

	if err := session.Run("/usr/bin/scp -qrt ./"); err != nil {
		return fmt.Errorf("Failed to run: %s", err.Error())
	}

	return nil
}

func (ss *ScpStorage) Pull(remotePath, localPath string) error {
	session, err := ss.connexion.NewSession()
	if err != nil {
		return fmt.Errorf("Failed to create session: %s", err.Error())
	}
	defer session.Close()

	var remoteFileBuffer bytes.Buffer
	session.Stdout = &remoteFileBuffer

	if err := session.Run(fmt.Sprintf("cat %s", remotePath)); err != nil {
		return fmt.Errorf("Failed to run: %s", err.Error())
	}

	err = ioutil.WriteFile(localPath, remoteFileBuffer.Bytes(), 0744)
	if err != nil {
		return err
	}

	return nil
}
