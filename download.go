package trousseau

import (
	"fmt"
	"github.com/crowdmob/goamz/aws"
	"github.com/oleiade/trousseau/dsn"
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

	s3Storage := NewS3Storage(awsAuth, dsn.Host, awsRegion)
    err := s3Storage.Connect()
	if err != nil {
		fmt.Errorf("Unable to connect to S3, have you set %s env var?",
			ENV_S3_BUCKET_KEY)
	}

	err = s3Storage.Pull(dsn.Path)
	if err != nil {
		return err
	}

	return nil
}

// downloadUsingScp executes the whole process of pulling
// the trousseau data store file from scp remote storage
// using the provided environment.
func DownloadUsingScp(dsn *dsn.Dsn, privateKey string) error {
	privateKeyContent, err := DecodePrivateKeyFromFile(privateKey)
	if err != nil {
		return err
	}

	keyChain := NewKeychain(privateKeyContent)
	scpStorage := NewScpStorage(dsn.Host,
		dsn.Port,
		dsn.Id,
		keyChain)
	err = scpStorage.Connect()
	if err != nil {
		return err
	}

	err = scpStorage.Pull(dsn.Path)
	if err != nil {
		return err
	}

	return nil
}
