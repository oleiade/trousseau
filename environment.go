package trousseau

import (
	"errors"
	"fmt"
	"github.com/oleiade/reflections"
	"os"
)

type Environment struct {
	RemoteHost     string
	RemotePort     string
	RemoteUser     string
	RemoteFilename string
	S3Bucket       string `env:"TROUSSEAU_S3_BUCKET"`
	SshPrivateKey  string `env:"TROUSSEAU_SSH_PRIVATE_KEY"`
	Password       string `env:"TROUSSEAU_PASSWORD"`
}

func NewEnvironment() *Environment {
	env := &Environment{
		RemoteHost:     "",
		RemotePort:     "22",
		RemoteUser:     "",
		RemoteFilename: "trousseau",
		S3Bucket:       "",
		SshPrivateKey:  gPrivateRsaKeyPath,
		Password:       "",
	}
	env.Load()

	return env
}

// Load method will try to load Environment struct
// tagged fields value from system environement. Every
// found values will override Environment field current
// value.
func (e *Environment) Load() error {
	var err error
	var envTags map[string]string

	envTags, err = reflections.Tags(*e, "env")
	if err != nil {
		return err
	}

	for field, tag := range envTags {
		envVar := os.Getenv(tag)

		if envVar != "" {
			err = reflections.SetField(e, field, envVar)
			if err != nil {
				return err
			}
		}
	}

	return nil
}

// OverrideWith will override the Environment values
// according to a provided key-value map.
func (e *Environment) OverrideWith(data map[string]string) error {
	for field, value := range data {
		has, err := reflections.HasField(*e, field)
		if !has {
			errMsg := fmt.Sprintf("No such field %s in Environement struct",
				field)
			return errors.New(errMsg)
		} else if err != nil {
			return err
		}

		if value != "" {
			reflections.SetField(e, field, value)
		}
	}

	return nil
}
