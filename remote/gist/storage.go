package gist

import (
    "github.com/google/go-github/github"
    "code.google.com/p/goauth2/oauth"
)

type GistStorage struct {
    connexion   *github.Client
    transport   *oauth.Transport

    Token       string
    User        string
}

func NewGistStorage(token string) *GistStorage {
    transport := &oauth.Transport{
        Token: &oauth.Token{AccessToken: token},
    }

    return &GistStorage{
        transport: transport,
        Token: token,
    }
}

func (gs *GistStorage) Connect() {
    gs.connexion = github.NewClient(gs.transport.Client())
}


func (gs *GistStorage) Push(localPath, remotePath string) error {
    gist := &github.Gist{
        Description: "Test",
        Public: false,
        files: map[github.GistFilename]GistFile{
            remotePath: &GistFile{
                Filename: localPath,
            },
        },
    }
}

func (gs *GistStorage) Pull(remotePath, localPath string) error {

}
