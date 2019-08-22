package ssh

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"strings"

	"golang.org/x/crypto/ssh"
)

type ScpStorage struct {
	host   string
	port   string
	client *ssh.Client

	PrivateKey string
	Password   string
	User       string
	Endpoint   string
}

func (ss *ScpStorage) Connect() error {
	var err error

	publicKeyAuth, err := ss.getPublicKeyAuthFrom(ss.PrivateKey)
	if err != nil {
		return fmt.Errorf("unable to get public key: %s", err.Error())
	}

	clientConfig := &ssh.ClientConfig{
		User: ss.User,
		Auth: []ssh.AuthMethod{
			ssh.Password(ss.Password),
			publicKeyAuth,
		},
	}

	ss.client, err = ssh.Dial("tcp", ss.Endpoint, clientConfig)
	if err != nil {
		return fmt.Errorf("Failed to dial: %s", err.Error())
	}

	return nil
}

func (ss *ScpStorage) Push(localPath, remotePath string) error {
	session, err := ss.client.NewSession()
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
	session, err := ss.client.NewSession()
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

func (ss *ScpStorage) getPublicKeyAuthFrom(path string) (ssh.AuthMethod, error) {
	key, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("unable to read private key: %v", err)
	}

	// Create the Signer for this private key.
	signer, err := ssh.ParsePrivateKey(key)
	if err != nil {
		return nil, fmt.Errorf("unable to parse private key: %v", err)
	}

	return ssh.PublicKeys(signer), nil
}

func NewScpStorage(host, port, user, password string, privateKey string) *ScpStorage {
	return &ScpStorage{
		PrivateKey: privateKey,
		Password:   password,
		User:       user,
		Endpoint:   strings.Join([]string{host, port}, ":"),
		host:       host,
		port:       port,
	}
}
