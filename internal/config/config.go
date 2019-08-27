package config

import (
	"encoding/json"
	"fmt"
	"io/ioutil"

	validation "github.com/go-ozzo/ozzo-validation"

	"github.com/BurntSushi/toml"
	"github.com/imdario/mergo"
	"github.com/joeshaw/envdecode"
	defaults "github.com/mcuadros/go-defaults"
)

// Config holds configuration for crate applications.
type Config struct {
	Filename    string `env:"TROUSSEAU_STORE_FILENAME" toml:"filename" default:".trousseau"`
	StorePath   string `env:"ENV_TROUSSEAU_STORE" toml:"store_path" default:".trousseau"`
	Passphrase  string `env:"ENV_PASSPHRASE_KEY"`
	MasterGPGID string `env:"ENV_MASTER_GPG_ID_KEY" toml:"master_gpg_id"`

	Encryption encryption `toml:"encryption"`
	Keyring    keyring    `toml:"keyring"`
}

type encryption struct {
	Type      string `env:"TROUSSEAU_ENCRYPTION_TYPE" toml:"type" default:"asymmetric"`
	Algorithm string `env:"TROUSSEAU_ENCRYPTION_ALGORITHM" toml:"algorithm" default:"gpg"`
}

type keyring struct {
	UserKey    string `env:"ENV_KEYRING_USER_KEY" toml:"user_key" default:"trousseau_user"`
	ServiceKey string `env:"ENV_KEYRING_SERVICE_KEY" toml:"service_key" default:"trousseau_service"`
}

// Load creates a Config object fullfiled with values extracted from
// the environment.
func Load(path string) (*Config, error) {
	config := new(Config)
	defaults.SetDefaults(config)

	if path != "" {
		d, err := ioutil.ReadFile(path)
		if err != nil {
			return nil, err
		}

		tomlConfig := config
		_, err = toml.Decode(string(d), tomlConfig)
		if err != nil {
			return nil, err
		}

		err = mergo.Merge(config, tomlConfig, mergo.WithOverride)
		if err != nil {
			return nil, err
		}

		err = config.Validate()
		if err != nil {
			return nil, fmt.Errorf("invalid values found while validating config loaded from TOML file")
		}
	}

	envConfig := new(Config)
	err := envdecode.Decode(envConfig)
	if err != nil && err != envdecode.ErrNoTargetFieldsAreSet {
		return nil, err
	}

	err = mergo.MergeWithOverwrite(config, envConfig)
	if err != nil {
		return nil, err
	}

	err = config.Validate()
	if err != nil {
		return nil, fmt.Errorf("invalid values found while validating config loaded from the env")
	}

	return config, nil
}

// Default initializes a Config instance with default values
func Default() *Config {
	config := new(Config)
	defaults.SetDefaults(config)
	return config
}

// Validate ensures Config instance contains valid values
func (c Config) Validate() error {
	err := validation.ValidateStruct(&c,
		validation.Field(&c.Filename),
		validation.Field(&c.StorePath),
		validation.Field(&c.Passphrase),
		validation.Field(&c.MasterGPGID),
		validation.Field(&c.Encryption),
		validation.Field(&c.Keyring),
	)
	if err != nil {
		return fmt.Errorf("invalid configuration provided")
	}

	return nil
}

// JSON serializes a Config object to a string.
func (c *Config) JSON() string {
	b, _ := json.Marshal(c)
	return string(b)
}

// String is a serialized JSON string representation
// of the Config values.
func (c *Config) String() string {
	return c.JSON()
}

func (e encryption) Validate() error {
	return validation.ValidateStruct(&e,
		validation.Field(&e.Type, validation.In("asymmetric", "symmetric")),
		validation.Field(&e.Algorithm, validation.In("gpg", "aes")),
	)
}

func (k keyring) Validate() error {
	return validation.ValidateStruct(&k,
		validation.Field(&k.UserKey),
		validation.Field(&k.ServiceKey),
	)
}
