package remote

import (
	"fmt"
	"io"
	"io/ioutil"
	"strings"
	"sync"

	"golang.org/x/crypto/ssh"
)

// SSHHandler holds a connexion to a remote server.
// It implements the Handler interface.
type SSHHandler struct {
	host   string
	port   string
	client *ssh.Client

	PrivateKey string
	Password   string
	User       string
	Endpoint   string
}

// NewSSHHandler generates a new SSHHandler
func NewSSHHandler(host, port, user, password string, privateKey string) *SSHHandler {
	return &SSHHandler{
		PrivateKey: privateKey,
		Password:   password,
		User:       user,
		Endpoint:   strings.Join([]string{host, port}, ":"),
		host:       host,
		port:       port,
	}
}

// Push reads the provided io.ReadSeeker and write its content to a
// file on the remote server.
func (h *SSHHandler) Push(remotePath string, r io.ReadSeeker) error {
	err := h.connect()
	if err != nil {
		return fmt.Errorf("unable to establish connexion to the remote server; reason: %s", err.Error())
	}

	session, err := h.client.NewSession()
	if err != nil {
		return fmt.Errorf("Failed to create session: %s", err.Error())
	}
	defer session.Close()

	wg := sync.WaitGroup{}
	wg.Add(2)
	uploadErr := make(chan error, 1)

	go func(ue chan error) {
		defer wg.Done()

		w, _ := session.StdinPipe()
		defer w.Close()

		var data []byte
		_, err := r.Read(data)
		if err != nil {
			ue <- fmt.Errorf("unable to read data intended for upload; reason: %s", err.Error())
		}

		fmt.Fprintln(w, "C0755", len(data), remotePath)
		fmt.Fprint(w, string(data))
		fmt.Fprint(w, "\x00")
		ue <- nil
	}(uploadErr)

	if err := session.Run("/usr/bin/scp -qrt ./"); err != nil {
		return fmt.Errorf("Failed to run: %s", err.Error())
	}

	wg.Wait()

	err = <-uploadErr
	if err != nil {
		return err
	}

	return nil
}

// Pull downloads the content of a file on the remote server,
// and writes it to the provided io.Writer.
func (h *SSHHandler) Pull(remotePath string, w io.Writer) error {
	err := h.connect()
	if err != nil {
		return fmt.Errorf("unable to establish connexion to the remote server; reason: %s", err.Error())
	}

	session, err := h.client.NewSession()
	if err != nil {
		return fmt.Errorf("Failed to create session: %s", err.Error())
	}
	defer session.Close()

	session.Stdout = w
	if err := session.Run(fmt.Sprintf("cat %s", remotePath)); err != nil {
		return fmt.Errorf("Failed to run: %s", err.Error())
	}

	return nil
}

func (h *SSHHandler) connect() error {
	var err error

	publicKeyAuth, err := h.getPublicKeyAuthFrom(h.PrivateKey)
	if err != nil {
		return fmt.Errorf("unable to get public key: %s", err.Error())
	}

	clientConfig := &ssh.ClientConfig{
		User: h.User,
		Auth: []ssh.AuthMethod{
			ssh.Password(h.Password),
			publicKeyAuth,
		},
	}

	h.client, err = ssh.Dial("tcp", h.Endpoint, clientConfig)
	if err != nil {
		return fmt.Errorf("Failed to dial: %s", err.Error())
	}

	return nil
}

func (h *SSHHandler) getPublicKeyAuthFrom(path string) (ssh.AuthMethod, error) {
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
