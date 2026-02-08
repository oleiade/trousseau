package main

import (
	"bytes"
	"io/ioutil"
	"log"
	"os"
	"path"

	"github.com/BurntSushi/toml"
	"github.com/oleiade/trousseau/internal/config"
	"github.com/oleiade/trousseau/internal/trousseau"
	"github.com/urfave/cli"
)

func Before(c *cli.Context) error {
	checkHelp(c)
	if c.GlobalBool("h") || c.GlobalBool("help") {
		return nil
	}
	checkConfig(c)
	trousseau.SetConfigPath(c.String("config"))
	updateStorePath(c)
	updateGnupgHome(c)
	updateCheckPassphrase(c)

	return nil
}

// checkHelp will print command or app help according to the
// provided context. It is used to bypass the gpg key check
// before the application runs. So users can print the help
// without selecting their master key.
func checkHelp(c *cli.Context) {
	if c.GlobalBool("h") || c.GlobalBool("help") {
		if len(c.Args()) >= 1 {
			cli.ShowCommandHelp(c, c.Args().First())
		} else {
			cli.ShowAppHelp(c)
		}
	}
}

// updateStorePath selects the default trousseau data store if
// none were provided on the command line
func updateStorePath(c *cli.Context) {
	if c.String("store") != "" {
		trousseau.SetStorePath(c.String("store"))
	}
}

func updateGnupgHome(c *cli.Context) {
	if c.String("gnupg-home") != "" {
		trousseau.GnupgHome = c.String("gnupg-home")
	}
}

func updateCheckPassphrase(c *cli.Context) {
	if c.GlobalBool("ask-passphrase") && !trousseau.AskPassphraseFlagCheck() {
		// This checks if the user is creating a store by
		// looking in c.Args()
		if c.Args().Get(0) == "create" {
			trousseau.AskPassphrase(true)
		} else {
			trousseau.AskPassphrase(false)
		}
	}
}

func checkConfig(c *cli.Context) {
	if _, err := os.Stat(c.String("config")); os.IsNotExist(err) {
		err := os.MkdirAll(path.Dir(c.String("config")), 0755)
		if err != nil {
			log.Fatalf("unable to create directory %s in order to store trousseau's config; reason: %s\n", c.String("config"), err.Error())
		}

		config := config.Default()
		buf := new(bytes.Buffer)
		if err := toml.NewEncoder(buf).Encode(config); err != nil {
			log.Fatalf("unable to encode trousseau's default configuration to toml; reason: %s\n", err.Error())
		}

		err = ioutil.WriteFile(c.String("config"), buf.Bytes(), 0600)
		if err != nil {
			log.Fatalf("unable to create trousseau's configuration file at %s; reason: %s\n", c.String("config"), err.Error())
		}
	}
}
