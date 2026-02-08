package main

import (
	"bytes"
	"context"
	"log"
	"os"
	"path"

	"github.com/BurntSushi/toml"
	"github.com/oleiade/trousseau/internal/config"
	"github.com/oleiade/trousseau/internal/trousseau"
	"github.com/urfave/cli/v3"
)

func Before(ctx context.Context, cmd *cli.Command) (context.Context, error) {
	checkHelp(cmd)
	if cmd.Bool("help") {
		return ctx, nil
	}
	checkConfig(cmd)
	trousseau.SetConfigPath(cmd.String("config"))
	updateStorePath(cmd)
	updateGnupgHome(cmd)
	updateCheckPassphrase(cmd)

	return ctx, nil
}

// checkHelp will print command or app help according to the
// provided context. It is used to bypass the gpg key check
// before the application runs. So users can print the help
// without selecting their master key.
func checkHelp(cmd *cli.Command) {
	if cmd.Bool("help") {
		if cmd.Args().Len() >= 1 {
			// In v3, we can't easily show subcommand help from Before.
			// The framework handles help display automatically.
			return
		}
	}
}

// updateStorePath selects the default trousseau data store if
// none were provided on the command line
func updateStorePath(cmd *cli.Command) {
	if cmd.String("store") != "" {
		trousseau.SetStorePath(cmd.String("store"))
	}
}

func updateGnupgHome(cmd *cli.Command) {
	if cmd.String("gnupg-home") != "" {
		trousseau.GnupgHome = cmd.String("gnupg-home")
	}
}

func updateCheckPassphrase(cmd *cli.Command) {
	if cmd.Bool("ask-passphrase") && !trousseau.AskPassphraseFlagCheck() {
		// This checks if the user is creating a store by
		// looking at the first argument
		if cmd.Args().Get(0) == "create" {
			trousseau.AskPassphrase(true)
		} else {
			trousseau.AskPassphrase(false)
		}
	}
}

func checkConfig(cmd *cli.Command) {
	if _, err := os.Stat(cmd.String("config")); os.IsNotExist(err) {
		err := os.MkdirAll(path.Dir(cmd.String("config")), 0755)
		if err != nil {
			log.Fatalf("unable to create directory %s in order to store trousseau's config; reason: %s\n", cmd.String("config"), err.Error())
		}

		cfg := config.Default()
		buf := new(bytes.Buffer)
		if err := toml.NewEncoder(buf).Encode(cfg); err != nil {
			log.Fatalf("unable to encode trousseau's default configuration to toml; reason: %s\n", err.Error())
		}

		err = os.WriteFile(cmd.String("config"), buf.Bytes(), 0600)
		if err != nil {
			log.Fatalf("unable to create trousseau's configuration file at %s; reason: %s\n", cmd.String("config"), err.Error())
		}
	}
}
