package main

import (
	"os"

	"github.com/codegangsta/cli"
	"github.com/oleiade/trousseau"
)

func main() {
	app := cli.NewApp()

	app.Name = "trousseau"
	app.Usage = "handles an encrypted keys store"
	app.Version = trousseau.TROUSSEAU_VERSION
	app.Commands = []cli.Command{
		CreateCommand(),
		PushCommand(),
		PullCommand(),
		ExportCommand(),
		ImportCommand(),
		ListRecipientsCommand(),
		AddRecipientCommand(),
		RemoveRecipientCommand(),
		SetCommand(),
		GetCommand(),
		RenameCommand(),
		DelCommand(),
		KeysCommand(),
		ShowCommand(),
		MetaCommand(),
		UpgradeCommand(),
	}
	app.Flags = []cli.Flag{
		VerboseFlag(),
		StoreFlag(),
	}
	app.Before = Before

	trousseau.Logger.Formatter = new(trousseau.RawFormatter)
	app.Run(os.Args)
}
