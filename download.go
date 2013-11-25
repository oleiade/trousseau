package trousseau

import (
	"fmt"
	"github.com/crowdmob/goamz/aws"
)

// downloadUsingS3 executes the whole process of pulling
// the trousseau data store file from s3 remote storage
// using the provided environment.
func DownloadUsingS3(bucket, remoteFilename, region string) error {
	awsAuth, err := aws.EnvAuth()
	if err != nil {
		return err
	}

    awsRegion, ok := aws.Regions[region]
    if !ok {
        return fmt.Errorf("Invalid aws region supplied %s", region)
    }

	s3Storage := NewS3Storage(awsAuth, bucket, awsRegion)
	err = s3Storage.Connect()
	if err != nil {
		fmt.Errorf("Unable to connect to S3, have you set %s env var?",
			ENV_S3_BUCKET_KEY)
	}

	err = s3Storage.Pull(remoteFilename)
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
