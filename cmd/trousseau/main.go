package main

import (
	"context"
	"os"
	"path/filepath"

	"github.com/oleiade/trousseau/internal/trousseau"

	"github.com/urfave/cli/v3"
)

func defaultConfigPath() string {
	configDir, err := os.UserConfigDir()
	if err != nil {
		configDir = os.Getenv("HOME")
	}
	return filepath.Join(configDir, "trousseau", "config.toml")
}

func main() {
	cmd := &cli.Command{
		Name:    "trousseau",
		Usage:   "Create, manage and share an encrypted data store",
		Version: trousseau.TROUSSEAU_VERSION,
		Authors: []any{"oleiade <tcrevon@gmail.com>"},
		Commands: []*cli.Command{
			CreateCommand(),
			SetCommand(),
			GetCommand(),
			RenameCommand(),
			DelCommand(),
			KeysCommand(),
			ShowCommand(),
			ExportCommand(),
			ImportCommand(),
			PushCommand(),
			PullCommand(),
			ListRecipientsCommand(),
			AddRecipientCommand(),
			RemoveRecipientCommand(),
			MetaCommand(),
			UpgradeCommand(),
		},
		Flags: []cli.Flag{
			&cli.StringFlag{
				Name:    "config",
				Aliases: []string{"c"},
				Value:   defaultConfigPath(),
				Usage:   "Filename of config file to override default lookup",
			},
			&cli.StringFlag{
				Name:    "store",
				Aliases: []string{"s"},
				Usage:   "Path to the trousseau data store to use",
			},
			&cli.BoolFlag{
				Name:  "ask-passphrase",
				Usage: "Have trousseu prompt user for passphrase",
			},
			&cli.BoolFlag{
				Name:  "verbose",
				Usage: "Set trousseau in verbose mode",
			},
			&cli.StringFlag{
				Name:  "gnupg-home",
				Usage: "Provide an alternate gnupg home",
			},
		},

		EnableShellCompletion: true,
		HideHelp:              false,
		HideVersion:           false,

		Before: Before,
	}

	cmd.Run(context.Background(), os.Args)
}
