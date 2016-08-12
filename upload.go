package trousseau

import (
	"fmt"

	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/dsn"
	"github.com/oleiade/trousseau/remote/gist"
	"github.com/oleiade/trousseau/remote/s3"
	"github.com/oleiade/trousseau/remote/ssh"
)

// uploadUsingS3 executes the whole process of pushing
// the trousseau data store file to s3 remote storage
// using the provided environment.
func UploadUsingS3(dsn *dsn.Dsn) error {
	awsAuth := aws.Auth{AccessKey: dsn.Id, SecretKey: dsn.Secret}

	awsRegion, ok := aws.Regions[dsn.Port]
	if !ok {
		return fmt.Errorf("Invalid aws region supplied %s", dsn.Port)
	}

	s3Storage := s3.NewS3Storage(awsAuth, dsn.Host, awsRegion)
	err := s3Storage.Connect()
	if err != nil {
		return fmt.Errorf("Unable to connect to S3")
	}

	err = s3Storage.Push(GetStorePath(), dsn.Path)
	if err != nil {
		return err
	}

	return nil
}

// uploadUsingScp executes the whole process of pushing
// the trousseau data store file to scp remote storage
// using the provided environment.
func UploadUsingScp(dsn *dsn.Dsn, privateKey string) (err error) {
	scpStorage := ssh.NewScpStorage(
		dsn.Host,
		dsn.Port,
		dsn.Id,
		dsn.Secret,
		privateKey,
	)

	err = scpStorage.Connect()
	if err != nil {
		return err
	}

	err = scpStorage.Push(GetStorePath(), dsn.Path)
	if err != nil {
		return err
	}

	return nil
}

// uploadUsingGist executes the whole process of pushing
// the trousseau data store file to gist remote storage
// using the provided dsn informations.
func UploadUsingGist(dsn *dsn.Dsn) (err error) {
	gistStorage := gist.NewGistStorage(dsn.Id, dsn.Secret)
	gistStorage.Connect()

	err = gistStorage.Push(GetStorePath(), dsn.Path)
	if err != nil {
		return err
	}

	return nil
}
