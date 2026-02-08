package remote

import (
	"context"
	"fmt"
	"io"

	"github.com/google/go-github/v68/github"
	"golang.org/x/oauth2"
)

// GistHandler holds a session to Github's Gist service.
// It implements the Handler interface.
type GistHandler struct {
	connexion *github.Client

	User string
}

// NewGistHandler generates a new GistHandler
func NewGistHandler(user, token string) *GistHandler {
	return &GistHandler{
		connexion: github.NewClient(
			oauth2.NewClient(
				context.Background(),
				oauth2.StaticTokenSource(&oauth2.Token{AccessToken: token}),
			),
		),
		User: user,
	}
}

// Connect starts the GistHandler connexion to Github Gists webservice
func (h *GistHandler) Connect() error {
	return nil
}

// Push reads the provided io.ReadSeeker and puts its content into a
// Github gist.
func (h *GistHandler) Push(filename string, r io.ReadSeeker) error {
	data, err := io.ReadAll(r)
	if err != nil {
		return fmt.Errorf("unable to read data store content; reason: %s", err.Error())
	}
	dataStr := string(data)

	// Create a gist representation ready to be
	// pushed over network
	public := false
	gist := &github.Gist{
		Public: &public,
		Files: map[github.GistFilename]github.GistFile{
			github.GistFilename(filename): {Content: &dataStr},
		},
	}

	// Push the gist to github
	_, _, err = h.connexion.Gists.Create(context.Background(), gist)
	if err != nil {
		return err
	}

	return nil
}

// Pull gets the content of a Github Gist, and writes it
// to the provided io.Writer.
func (h *GistHandler) Pull(gistname string, w io.Writer) error {
	// Fetch the user's gists list
	gists, _, err := h.connexion.Gists.List(context.Background(), h.User, nil)
	if err != nil {
		return err
	}

	// Find a gist containing trousseau data store
	var gist *github.Gist
	for _, g := range gists {
		for k := range g.Files {
			if string(k) == gistname {
				gist = g
				break
			}
		}

		if gist != nil {
			break
		}
	}

	// Download the gist file content
	gist, _, err = h.connexion.Gists.Get(context.Background(), *gist.ID)
	if err != nil {
		return fmt.Errorf("unable to download gist from Github; reason: %s", err.Error())
	}

	// Write the downloaded file content to the local trousseau
	// data store file
	content := []byte(*gist.Files[github.GistFilename(gistname)].Content)
	_, err = w.Write(content)
	if err != nil {
		return fmt.Errorf("unable to write gist file content; reason: %s", err.Error())
	}

	return nil
}
