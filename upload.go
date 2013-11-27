package trousseau

import (
	"fmt"
	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/dsn"
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

	s3Storage := NewS3Storage(awsAuth, dsn.Host, awsRegion)
    err := s3Storage.Connect()
	if err != nil {
		return fmt.Errorf("Unable to connect to S3, have you set %s env var?",
			ENV_S3_BUCKET_KEY)
	}

	err = s3Storage.Push(dsn.Path)
	if err != nil {
		return err
	}

	return nil
}

// uploadUsingScp executes the whole process of pushing
// the trousseau data store file to scp remote storage
// using the provided environment.
func uploadUsingScp(privateKey, remoteFilename, host, port, user string) error {
	privateKeyContent, err := DecodePrivateKeyFromFile(privateKey)
	if err != nil {
		return err
	}

	keyChain := NewKeychain(privateKeyContent)
	scpStorage := NewScpStorage(host,
		port,
		user,
		keyChain)
	err = scpStorage.Connect()
	if err != nil {
		return err
	}

	err = scpStorage.Push(remoteFilename)
	if err != nil {
		return err
	}

	return nil
}
