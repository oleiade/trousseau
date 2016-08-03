package main

import (
	"os"

	"github.com/codegangsta/cli"
	"github.com/oleiade/trousseau"
)

func main() {
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
		cli.BoolFlag{
			Name:  "verbose",
			Usage: "Set trousseau in verbose mode",
		},
		cli.StringFlag{
			Name:  "store, s",
			Usage: "Path to the trousseau data store to use",
		},
		cli.StringFlag{
			Name:  "gnupg-home",
			Usage: "Provide an alternate gnupg home",
		},
		cli.BoolFlag{
			Name:  "ask-passphrase",
			Usage: "Have trousseu prompt user for passphrase",
		},
	}

	app.Before = Before
	app.Run(os.Args)
}
