package main

import (
	"os"

	"github.com/codegangsta/cli"
	trousseau_cli "github.com/oleiade/trousseau/cli"
	"github.com/oleiade/trousseau"
)

func main() {
	app := cli.NewApp()

	app.Name = "trousseau"
	app.Usage = "handles an encrypted keys store"
	app.Version = trousseau.TROUSSEAU_VERSION
	app.Commands = []cli.Command{
		trousseau_cli.CreateCommand(),
		trousseau_cli.PushCommand(),
		trousseau_cli.PullCommand(),
		trousseau_cli.ExportCommand(),
		trousseau_cli.ImportCommand(),
		trousseau_cli.ListRecipientsCommand(),
		trousseau_cli.AddRecipientCommand(),
		trousseau_cli.RemoveRecipientCommand(),
		trousseau_cli.SetCommand(),
		trousseau_cli.GetCommand(),
		trousseau_cli.RenameCommand(),
		trousseau_cli.DelCommand(),
		trousseau_cli.KeysCommand(),
		trousseau_cli.ShowCommand(),
		trousseau_cli.MetaCommand(),
		trousseau_cli.UpgradeCommand(),
	}
	app.Flags = []cli.Flag{
		trousseau_cli.VerboseFlag(),
		trousseau_cli.StoreFlag(),
	}
	app.Before = trousseau_cli.Before

	trousseau.Logger.Formatter = new(trousseau.RawFormatter)
	app.Run(os.Args)
}
