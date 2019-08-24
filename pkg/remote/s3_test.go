package remote

import (
	"reflect"
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
		want    *S3Handler
		wantErr bool
	}{
		{
			name:    "it should fail when AWS_ACCESS_KEY_ID is not present in the environment",
			args:    args{region: "foo", bucket: "bar"},
			want:    nil,
			wantErr: true,
		},
		{
			name:    "it should fail when AWS_SECRET_ACCESS_KEY is not present in the environment",
			args:    args{region: "foo", bucket: "bar"},
			want:    nil,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := NewS3Handler(tt.args.region, tt.args.bucket)
			if (err != nil) != tt.wantErr {
				t.Errorf("NewS3Handler() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("NewS3Handler() = %v, want %v", got, tt.want)
			}
		})
	}
}
