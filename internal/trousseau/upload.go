package trousseau

import (
	"fmt"

	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/pkg/remote/gist"
	"github.com/oleiade/trousseau/pkg/remote/s3"
	"github.com/oleiade/trousseau/pkg/remote/ssh"
)

// Uploader is an interface representing the capacity to upload
// files to remote services or servers.
type Uploader interface {
	Upload(path string) error
}

// S3Uploader allows uploading files to Amazon S3 service.
// It implements the Uploader interface.
type S3Uploader struct {
	storage *s3.S3Storage
}

// NewS3Uploader generates a S3 Uploader
func NewS3Uploader(region, accessKey, secretKey, bucket string) (*S3Uploader, error) {
	AWSRegion, ok := aws.Regions[region]
	if !ok {
		return nil, fmt.Errorf("Invalid aws region supplied %s", region)
	}

	uploader := &S3Uploader{
		storage: s3.NewS3Storage(
			aws.Auth{AccessKey: accessKey, SecretKey: secretKey},
			bucket,
			AWSRegion,
		),
	}

	return uploader, nil
}

// Upload executes the whole process of pushing
// the trousseau data store file to s3 remote storage
// using the provided environment.
func (s *S3Uploader) Upload(path string) error {
	err := s.storage.Connect()
	if err != nil {
		return fmt.Errorf("Unable to connect to S3")
	}

	err = s.storage.Push(GetStorePath(), path)
	if err != nil {
		return err
	}

	return nil
}

// SCPUploader allows uploading files to a remote server using SSH.
// It implements the Uploader interface.
type SCPUploader struct {
	storage *ssh.ScpStorage
}

// NewSCPUploader generates a SCP Uploader
func NewSCPUploader(host, port, user, password, privateKey string) *SCPUploader {
	return &SCPUploader{
		storage: ssh.NewScpStorage(
			host,
			port,
			user,
			password,
			privateKey,
		),
	}
}

// Upload executes the whole process of pushing
// the trousseau data store file to scp remote storage
// using the provided environment.
func (s *SCPUploader) Upload(path string) (err error) {
	err = s.storage.Connect()
	if err != nil {
		return err
	}

	err = s.storage.Push(GetStorePath(), path)
	if err != nil {
		return err
	}

	return nil
}

// GistUploader allows uploading files to Github's Gist service.
// It implements the Uploader interface.
type GistUploader struct {
	storage *gist.GistStorage
}

// NewGistUploader generates a gist uploader
func NewGistUploader(user, token string) *GistUploader {
	return &GistUploader{
		storage: gist.NewGistStorage(user, token),
	}
}

// Upload executes the whole process of pushing
// the trousseau data store file to gist remote storage
// using the provided dsn informations.
func (g *GistUploader) Upload(path string) error {
	g.storage.Connect()

	err := g.storage.Push(GetStorePath(), path)
	if err != nil {
		return err
	}

	return nil
}
