package remote

import (
	"testing"
)

func TestNewS3Handler(t *testing.T) {
	type args struct {
		region string
		bucket string
	}

	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name:    "it should succeed even without AWS credentials (resolved lazily in SDK v2)",
			args:    args{region: "us-east-1", bucket: "bar"},
			wantErr: false,
		},
		{
			name:    "it should succeed with a valid region and bucket",
			args:    args{region: "eu-west-1", bucket: "my-bucket"},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := NewS3Handler(tt.args.region, tt.args.bucket)
			if (err != nil) != tt.wantErr {
				t.Errorf("NewS3Handler() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr && got == nil {
				t.Errorf("NewS3Handler() returned nil, want non-nil handler")
			}
			if !tt.wantErr && got != nil && got.Bucket != tt.args.bucket {
				t.Errorf("NewS3Handler().Bucket = %v, want %v", got.Bucket, tt.args.bucket)
			}
		})
	}
}
