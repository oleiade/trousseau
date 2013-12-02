package trousseau

import (
	"fmt"
	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/dsn"
	"github.com/oleiade/trousseau/remote/s3"
	"github.com/oleiade/trousseau/remote/ssh"
)

// uploadUsingS3 executes the whole process of pushing
// the trousseau data store file to s3 remote storage
// using the provided environment.
func uploadUsingS3(dsn *dsn.Dsn) error {
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

	err = s3Storage.Push(gStorePath, dsn.Path)
	if err != nil {
		return err
	}

	return nil
}

// uploadUsingScp executes the whole process of pushing
// the trousseau data store file to scp remote storage
// using the provided environment.
func uploadUsingScp(dsn *dsn.Dsn, privateKey string) (err error) {
	keychain := new(ssh.Keychain)
	keychain.AddPEMKey(privateKey)

	scpStorage := ssh.NewScpStorage(dsn.Host,
		dsn.Port,
		dsn.Id,
		dsn.Secret,
		keychain)
	err = scpStorage.Connect()
	if err != nil {
		return err
	}

	err = scpStorage.Push(gStorePath, dsn.Path)
	if err != nil {
		return err
	}

	return nil
}
