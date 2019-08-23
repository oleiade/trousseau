package trousseau

import (
	"fmt"

	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/pkg/remote/gist"
	"github.com/oleiade/trousseau/pkg/remote/s3"
	"github.com/oleiade/trousseau/pkg/remote/ssh"
)

// Downloader is an interface representing the capacity to upload
// files to remote services or servers.
type Downloader interface {
	Download(path string) error
}

// S3Downloader allows downloading files to Amazon S3 service.
// It implements the Downloader interface.
type S3Downloader struct {
	storage *s3.S3Storage
}

// NewS3Downloader generates a S3 Downloader
func NewS3Downloader(region, accessKey, secretKey, bucket string) (*S3Downloader, error) {
	AWSRegion, ok := aws.Regions[region]
	if !ok {
		return nil, fmt.Errorf("Invalid aws region supplied %s", region)
	}

	downloader := &S3Downloader{
		storage: s3.NewS3Storage(
			aws.Auth{AccessKey: accessKey, SecretKey: secretKey},
			bucket,
			AWSRegion,
		),
	}

	return downloader, nil
}

// Download executes the whole process of pulling
// the trousseau data store file from s3 remote storage
// using the provided environment.
func (s *S3Downloader) Download(path string) error {
	err := s.storage.Connect()
	if err != nil {
		return fmt.Errorf("Unable to connect to S3")
	}

	err = s.storage.Pull(path, GetStorePath())
	if err != nil {
		return err
	}

	return nil
}

// SCPDownloader allows downloading files to a remote server using SSH.
// It implements the Downloader interface.
type SCPDownloader struct {
	storage *ssh.ScpStorage
}

// NewSCPDownloader generates a SCP Downloader
func NewSCPDownloader(host, port, user, password, privateKey string) *SCPDownloader {
	return &SCPDownloader{
		storage: ssh.NewScpStorage(
			host,
			port,
			user,
			password,
			privateKey,
		),
	}
}

// Download executes the whole process of downloading
// the trousseau data store file to scp remote storage
// using the provided environment.
func (s *SCPDownloader) Download(path string) (err error) {
	err = s.storage.Connect()
	if err != nil {
		return err
	}

	err = s.storage.Pull(path, GetStorePath())
	if err != nil {
		return err
	}

	return nil
}

// GistDownloader allows downloading files to Github's Gist service.
// It implements the Downloader interface.
type GistDownloader struct {
	storage *gist.GistStorage
}

// NewGistDownloader generates a gist Downloader
func NewGistDownloader(user, token string) *GistDownloader {
	return &GistDownloader{
		storage: gist.NewGistStorage(user, token),
	}
}

// Download executes the whole process of downloading
// the trousseau data store file to gist remote storage
// using the provided dsn informations.
func (g *GistDownloader) Download(path string) error {
	g.storage.Connect()

	err := g.storage.Pull(path, GetStorePath())
	if err != nil {
		return err
	}

	return nil
}
