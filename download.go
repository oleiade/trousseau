package trousseau

import (
	"fmt"
	"launchpad.net/goamz/aws"
)

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

	err = s3Storage.Pull(env.S3Filename)
	if err != nil {
		return err
	}

	return nil
}

func DownloadUsingScp(env *Environment) error {
	return nil
}