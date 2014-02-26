package gist

import (
    "io/ioutil"
    "github.com/google/go-github/github"
    "code.google.com/p/goauth2/oauth"
)

type GistStorage struct {
    connexion   *github.Client
    transport   *oauth.Transport

    Token       string
    User        string
}

func NewGistStorage(user, token string) *GistStorage {
    transport := &oauth.Transport{
        Token: &oauth.Token{AccessToken: token},
    }

    return &GistStorage{
        transport: transport,
        Token: token,
        User: user,
    }
}

func (gs *GistStorage) Connect() {
    gs.connexion = github.NewClient(gs.transport.Client())
}

// Push exports the local encrypted trousseau data store to
// a Github private gist
func (gs *GistStorage) Push(localPath, remotePath string) (err error) {
    var public bool = false

    // Read the encrypted data store content
    fileBytes, err := ioutil.ReadFile(localPath)
    if err != nil {
        return err
    }
    fileContent := string(fileBytes)

    // Build a Gist file store
    files := map[github.GistFilename]github.GistFile{
        github.GistFilename(remotePath): github.GistFile{
            Content: &fileContent,
        },
    }

    // Create a gist representation ready to be
    // pushed over network
    gist := &github.Gist{
        Public: &public,
        Files: files,
    }

    // Push the gist to github
    _, _, err = gs.connexion.Gists.Create(gist)
    if err != nil {
        return err
    }

    return nil
}

// Pull imports the encrypted trousseau data store from
// a Github gist
func (gs *GistStorage) Pull(remotePath, localPath string) (err error) {
    var gist *github.Gist

    // Fetch the user's gists list
    gists, _, err := gs.connexion.Gists.List(gs.User, nil)
    if err != nil {
        return err
    }

    // Find a gist containing trousseau data store
    for _, g := range gists {
        for k, _ := range g.Files {
            if string(k) == remotePath {
                gist = &g
                break
            }
        }

        if gist != nil {
            break
        }
    }

    // Download the gist file content
    gist, _, err = gs.connexion.Gists.Get(*gist.ID)

    // Write the downloaded file content to the local trousseau
    // data store file
    fileContent := []byte(*gist.Files[github.GistFilename(remotePath)].Content)
    err = ioutil.WriteFile(localPath, fileContent, 0600)
    if err != nil {
        return err
    }


    return nil
}
