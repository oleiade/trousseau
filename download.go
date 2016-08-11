package trousseau

import (
	"fmt"

	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/dsn"
	"github.com/oleiade/trousseau/remote/gist"
	"github.com/oleiade/trousseau/remote/s3"
	"github.com/oleiade/trousseau/remote/ssh"
)

// downloadUsingS3 executes the whole process of pulling
// the trousseau data store file from s3 remote storage
// using the provided environment.
func DownloadUsingS3(dsn *dsn.Dsn) error {
	awsAuth := aws.Auth{AccessKey: dsn.Id, SecretKey: dsn.Secret}

	awsRegion, ok := aws.Regions[dsn.Port]
	if !ok {
		return fmt.Errorf("Invalid aws region supplied %s", dsn.Port)
	}

	s3Storage := s3.NewS3Storage(awsAuth, dsn.Host, awsRegion)
	err := s3Storage.Connect()
	if err != nil {
		fmt.Errorf("Unable to connect to S3")
	}

	err = s3Storage.Pull(dsn.Path, GetStorePath())
	if err != nil {
		return err
	}

	return nil
}

// downloadUsingScp executes the whole process of pulling
// the trousseau data store file from scp remote storage
// using the provided environment.
func DownloadUsingScp(dsn *dsn.Dsn, privateKey string) (err error) {
	scpStorage := ssh.NewScpStorage(dsn.Host,
		dsn.Port,
		dsn.Id,
		dsn.Secret,
		privateKey)
	err = scpStorage.Connect()
	if err != nil {
		return err
	}

	err = scpStorage.Pull(dsn.Path, GetStorePath())
	if err != nil {
		return err
	}

	return nil
}

// downloadUsingGist executes the whole process of pulling
// the trousseau data store file from gist remote storage
// using the provided scheme informations.
func DownloadUsingGist(dsn *dsn.Dsn) (err error) {
	gistStorage := gist.NewGistStorage(dsn.Id, dsn.Secret)
	gistStorage.Connect()

	err = gistStorage.Pull(dsn.Path, GetStorePath())
	if err != nil {
		return err
	}

	return nil
}
