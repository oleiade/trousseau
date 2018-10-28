package trousseau

import (
	"encoding/json"
	"fmt"

	"github.com/go-yaml/yaml"
	"github.com/naoina/toml"
)

type TrousseauFormatOutput interface {
	FormatOutput() error
}

type JSONFORMAT struct {
	kv KVStore
}
type YMLFORMAT struct {
	kv KVStore
}
type TOMLFORMAT struct {
	kv KVStore
}
type DEFUALTFORMAT struct {
	kv KVStore
}

func (format JSONFORMAT) FormatOutput() error {
	jsonFormat, err := json.Marshal(format.kv)
	if err != nil {
		return err
	}
	fmt.Println(string(jsonFormat))
	return nil
}

func (format YMLFORMAT) FormatOutput() error {
	ymlFormat, err := yaml.Marshal(format.kv)
	if err != nil {
		return err
	}
	fmt.Println(string(ymlFormat))
	return nil
}

func (format TOMLFORMAT) FormatOutput() error {
	tomlFormat, err := toml.Marshal(format.kv)
	if err != nil {
		return err
	}
	fmt.Println(string(tomlFormat))
	return nil
}

func (format DEFUALTFORMAT) FormatOutput() error {
	items := format.kv.Items()
	for k, v := range items {
		fmt.Println(fmt.Sprintf("%s : %s", k, v.(string)))
	}
	return nil
}

func PrintOutput(tfo TrousseauFormatOutput) error {
	return tfo.FormatOutput()
}

func ShowOutput(outputFormat string, kv KVStore) error {
	jsonFormatStrings := []string{"json"}
	yamlFormatStrings := []string{"yml", "yaml"}
	tomlFormatStrings := []string{"toml", "tml"}
	for _, formatString := range jsonFormatStrings {
		if outputFormat == formatString {
			return PrintOutput(JSONFORMAT{kv})
		}
	}
	for _, formatString := range yamlFormatStrings {
		if outputFormat == formatString {
			return PrintOutput(YMLFORMAT{kv})
		}
	}
	for _, formatString := range tomlFormatStrings {
		if outputFormat == formatString {
			return PrintOutput(TOMLFORMAT{kv})
		}
	}
	return PrintOutput(DEFUALTFORMAT{kv})
}
