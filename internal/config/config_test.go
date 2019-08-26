package config

import (
	"io/ioutil"
	"log"
	"os"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestLoad(t *testing.T) {
	type args struct {
		tomldata string
		env      map[string]string
	}
	tests := []struct {
		name    string
		args    args
		want    *Config
		wantErr bool
	}{
		{
			name:    "without env or file provided default values are used",
			args:    args{``, map[string]string{}},
			want:    func() *Config { return Default() }(),
			wantErr: false,
		},
		{
			name: "env values override defaults",
			args: args{``, map[string]string{"TROUSSEAU_STORE_FILENAME": "test"}},
			want: func() *Config {
				c := Default()
				c.Filename = "test"
				return c
			}(),
			wantErr: false,
		},
		{
			name: "env values override nested defaults",
			args: args{
				``,
				map[string]string{
					"TROUSSEAU_ENCRYPTION_TYPE": "symmetric",
				},
			},
			want: func() *Config {
				c := Default()
				c.Encryption.Type = "symmetric"
				return c
			}(),
			wantErr: false,
		},
		{
			name: "toml values override defaults",
			args: args{
				`filename = "test"`,
				map[string]string{},
			},
			want: func() *Config {
				c := Default()
				c.Filename = "test"
				return c
			}(),
			wantErr: false,
		},
		{
			name: "toml values override nested defaults",
			args: args{
				`[encryption]
				 type = "symmetric"
				`,
				map[string]string{},
			},
			want: func() *Config {
				c := Default()
				c.Encryption.Type = "symmetric"
				return c
			}(),
			wantErr: false,
		},
		{
			name: "env values precede toml values",
			args: args{
				`filename = "toml-test"`,
				map[string]string{
					"TROUSSEAU_STORE_FILENAME": "env-test",
				},
			},
			want: func() *Config {
				c := Default()
				c.Filename = "env-test"
				return c
			}(),
			wantErr: false,
		},
		{
			name: "env values precede toml scoped values",
			args: args{
				`[encryption]
				 type = "asymmetric"
				`,
				map[string]string{
					"TROUSSEAU_ENCRYPTION_TYPE": "symmetric",
				},
			},
			want: func() *Config {
				c := Default()
				c.Encryption.Type = "symmetric"
				return c
			}(),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// If any env values are provided, set them in the
			// environment, and defer their removal at the end of
			// the function's run.
			if len(tt.args.env) != 0 {
				for k, v := range tt.args.env {
					os.Setenv(k, v)
				}
			}

			var testFilePath string
			if len(tt.args.tomldata) != 0 {
				f, err := ioutil.TempFile("", "crate_config_test")
				if err != nil {
					log.Fatal("unable to create test file; reason: ", err.Error())
				}
				testFilePath = f.Name()

				if _, err := f.Write([]byte(tt.args.tomldata)); err != nil {
					log.Fatal("unable to write test content to test file; reason: ", err.Error())
				}
				if err := f.Close(); err != nil {
					log.Fatal("unable to close test file; reason: ", err.Error())
				}
			}

			got, err := Load(testFilePath)
			if (err != nil) != tt.wantErr {
				t.Errorf("Load() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(tt.want, got); diff != "" {
				t.Errorf("MakeGatewayInfo() mismatch (-want +got):\n%s", diff)
			}

			// CLEANUP
			for k := range tt.args.env {
				os.Unsetenv(k)
			}

			if testFilePath != "" {
				os.Remove(testFilePath)
			}
		})
	}
}

func TestDefault(t *testing.T) {
	tests := []struct {
		name string
		want *Config
	}{
		{
			"Config default values are correct",
			&Config{
				Filename:    ".trousseau",
				StorePath:   ".trousseau",
				Passphrase:  "",
				MasterGPGID: "",
				Encryption: encryption{
					Type:      "asymmetric",
					Algorithm: "gpg",
				},
				Keyring: keyring{
					UserKey:    "trousseau_user",
					ServiceKey: "trousseau_service",
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if diff := cmp.Diff(tt.want, Default()); diff != "" {
				t.Errorf("MakeGatewayInfo() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
