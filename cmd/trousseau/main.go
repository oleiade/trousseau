package main

import (
	"os"
	"path"

	"github.com/OpenPeeDeeP/xdg"
	"github.com/oleiade/trousseau/internal/trousseau"

	"github.com/urfave/cli"
)

func main() {
	cli.VersionFlag = cli.BoolFlag{
		Name:  "version, v",
		Usage: "print only the executable's version",
	}

	app := cli.NewApp()

	app.Name = "trousseau"
	app.Author = "oleiade"
	app.Email = "tcrevon@gmail.com"
	app.Usage = "Create, manage and share an encrypted data store"
	app.Version = trousseau.TROUSSEAU_VERSION
	app.Commands = []cli.Command{
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
	}
	app.Flags = []cli.Flag{
		cli.StringFlag{
			Name:  "config, c",
			Value: path.Join(xdg.ConfigHome(), "trousseau", "config.toml"),
			Usage: "Filename of config file to override default lookup",
		},
		cli.StringFlag{
			Name:  "store, s",
			Usage: "Path to the trousseau data store to use",
		},
		cli.BoolFlag{
			Name:  "ask-passphrase",
			Usage: "Have trousseu prompt user for passphrase",
		},
		cli.BoolFlag{
			Name:  "verbose",
			Usage: "Set trousseau in verbose mode",
		},
		cli.StringFlag{
			Name:  "gnupg-home",
			Usage: "Provide an alternate gnupg home",
		},
	}

	app.EnableBashCompletion = true
	app.HideHelp = false
	app.HideVersion = false

	app.Before = Before
	app.Run(os.Args)
}
