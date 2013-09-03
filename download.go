package trousseau

import (
	"fmt"
	"launchpad.net/goamz/aws"
)

// downloadUsingS3 executes the whole process of pulling
// the trousseau data store file from s3 remote storage
// using the provided environment.
func DownloadUsingS3(env *Environment) error {
	awsAuth, err := aws.EnvAuth()
	if err != nil {
		return err
	}

	s3Storage := NewS3Storage(awsAuth, env.S3Bucket, aws.EUWest)
	err = s3Storage.Connect()
	if err != nil {
		fmt.Errorf("Unable to connect to S3, have you set %s and %s env vars?",
			"TROUSSEAU_S3_FILENAME",
			"TROUSSEAU_S3_BUCKET")
	}

	err = s3Storage.Pull(env.RemoteFilename)
	if err != nil {
		return err
	}

	return nil
}

// downloadUsingScp executes the whole process of pulling
// the trousseau data store file from scp remote storage
// using the provided environment.
func DownloadUsingScp(env *Environment) error {
	privateKeyContent, err := DecodePrivateKeyFromFile(env.SshPrivateKey)
	if err != nil {
		return err
	}

	keyChain := NewKeychain(privateKeyContent)
	scpStorage := NewScpStorage(env.RemoteHost,
		env.RemotePort,
		env.RemoteUser,
		keyChain)
	err = scpStorage.Connect()
	if err != nil {
		return err
	}

	err = scpStorage.Pull(env.RemoteFilename)
	if err != nil {
		return err
	}

	return nil
}
