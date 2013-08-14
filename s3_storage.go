package trousseau

import (
	"errors"
	"fmt"
	"io/ioutil"
	"launchpad.net/goamz/aws"
	"launchpad.net/goamz/s3"
)

// S3Storage is an implementation of the RemoteStorage
// interface to be able to push/pull trousseau to a
// amazon S3 bucket.
type S3Storage struct {
	connexion  *s3.Bucket
	AwsAuth    aws.Auth
	BucketName string
	Region     aws.Region
}

func NewS3Storage(auth aws.Auth, bucketName string, region aws.Region) *S3Storage {
	s3Storage := S3Storage{
		AwsAuth:    auth,
		BucketName: bucketName,
		Region:     region,
	}

	return &s3Storage
}

func (ss *S3Storage) Connect() error {
	if ss.BucketName == "" {
		return errors.New("S3 bucket name mandatory to establish a connection")
	}

	s3Conn := s3.New(ss.AwsAuth, ss.Region)
	ss.connexion = s3Conn.Bucket(ss.BucketName)

	return nil
}

func (ss *S3Storage) Push(remoteName string) error {
	data, err := ioutil.ReadFile(gStorePath)
	if err != nil {
		return errors.New("Cannot push trousseau: Store file does not exist")
	}

	err = ss.connexion.Put(remoteName, data, "text/plain", s3.BucketOwnerFull)
	if err != nil {
		errMsg := "Unable to push trousseau file to S3: "

		if remoteName == "" {
			errMsg += fmt.Sprintf("Make sure you've set %s and %s env vars.",
				"TROUSSEAU_S3_FILENAME",
				"TROUSSEAU_S3_BUCKET")
		} else {
			errMsg += err.Error()
		}

		return errors.New(errMsg)
	}

	return nil
}

func (ss *S3Storage) Pull(remoteName string) error {
	data, err := ss.connexion.Get(remoteName)
	if err != nil {
		errMsg := "Unable to pull trousseau file from S3: "

		if remoteName == "" {
			errMsg += fmt.Sprintf("Make sure you've set %s and %s env vars.",
				"TROUSSEAU_S3_FILENAME",
				"TROUSSEAU_S3_BUCKET")
		} else {
			errMsg += err.Error()
		}

		return errors.New(errMsg)
	}

	// Write pulled json to trousseau file
	err = ioutil.WriteFile(gStorePath, data, 0764)
	if err != nil {
		errMsg := "Your trousseau installation seems to be unconfigured. "
		errMsg += "Please make sure you run trousseau configure before pulling"
		return errors.New(errMsg)
	}

	return nil
}
