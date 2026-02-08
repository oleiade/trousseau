package remote

import (
	"context"
	"fmt"
	"io"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/aws/aws-sdk-go-v2/service/s3/types"
)

// S3Handler holds a session to the Amazon S3 webservice.
// It implements the Handler interface.
type S3Handler struct {
	Client *s3.Client
	Bucket string
}

// NewS3Handler generates a S3Handler.
func NewS3Handler(region, bucket string) (*S3Handler, error) {
	cfg, err := config.LoadDefaultConfig(context.Background(),
		config.WithRegion(region),
	)
	if err != nil {
		return nil, fmt.Errorf("unable to load AWS config: %w", err)
	}

	return &S3Handler{
		Client: s3.NewFromConfig(cfg),
		Bucket: bucket,
	}, nil
}

// Connect starts the S3Handler session to the Amazon S3 webservice.
func (h *S3Handler) Connect() error {
	return nil
}

// Push reads the provided io.ReadSeeker and puts its content into an
// Amazon S3 object.
func (h *S3Handler) Push(key string, r io.ReadSeeker) error {
	_, err := h.Client.PutObject(context.Background(), &s3.PutObjectInput{
		Bucket:               aws.String(h.Bucket),
		Key:                  aws.String(key),
		ACL:                  types.ObjectCannedACLPrivate,
		Body:                 r,
		ContentDisposition:   aws.String("attachment"),
		ServerSideEncryption: types.ServerSideEncryptionAes256,
	})

	return err
}

// Pull gets the content of an Amazon S3 object, and writes it
// to the provided io.Writer.
func (h *S3Handler) Pull(key string, w io.Writer) error {
	results, err := h.Client.GetObject(context.Background(), &s3.GetObjectInput{
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
