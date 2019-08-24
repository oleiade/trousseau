package remote

import (
	"fmt"
	"io"
	"os"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/s3"
)

// S3Handler holds a session to the Amazon S3 webservice.
// It implements the Handler interface.
type S3Handler struct {
	Session *session.Session
	Bucket  string
}

// NewS3Handler generates a S3Handler.
func NewS3Handler(region, bucket string) (*S3Handler, error) {
	if os.Getenv("AWS_ACCESS_KEY_ID") == "" {
		return nil, fmt.Errorf("environment variable AWS_ACCESS_KEY_ID is not set")
	}

	if os.Getenv("AWS_SECRET_ACCESS_KEY") == "" {
		return nil, fmt.Errorf("environment variable AWS_SECRET_ACCESS_KEY is not set")
	}

	session, err := session.NewSession(&aws.Config{Region: aws.String(region)})
	if err != nil {
		return nil, err
	}

	return &S3Handler{
		Session: session,
		Bucket:  bucket,
	}, nil
}

// Connect starts the S3Handler session to the Amazon S3 webservice.
func (h *S3Handler) Connect() error {
	return nil
}

// Push reads the provided io.ReadSeeker and puts its content into an
// Amazon S3 object.
func (h *S3Handler) Push(key string, r io.ReadSeeker) error {
	_, err := s3.New(h.Session).PutObject(&s3.PutObjectInput{
		Bucket:               aws.String(h.Bucket),
		Key:                  aws.String(key),
		ACL:                  aws.String("private"),
		Body:                 r,
		ContentDisposition:   aws.String("attachment"),
		ServerSideEncryption: aws.String("AES256"),
	})

	return err
}

// Pull gets the content of an Amazon S3 object, and writes it
// to the provided io.Writer.
func (h *S3Handler) Pull(key string, w io.Writer) error {
	results, err := s3.New(h.Session).GetObject(&s3.GetObjectInput{
		Bucket: aws.String(h.Bucket),
		Key:    aws.String(key),
	})
	if err != nil {
		return err
	}
	defer results.Body.Close()

	if _, err := io.Copy(w, results.Body); err != nil {
		return err
	}
	return nil
}
