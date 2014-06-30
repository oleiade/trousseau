package main

import (
	"github.com/codegangsta/cli"
	trousseau_cli "github.com/oleiade/trousseau/cli"
	"github.com/oleiade/trousseau/trousseau"
	"os"
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
		trousseau_cli.AddRecipientCommand(),
		trousseau_cli.RemoveRecipientCommand(),
		trousseau_cli.SetCommand(),
		trousseau_cli.GetCommand(),
		trousseau_cli.DelCommand(),
		trousseau_cli.KeysCommand(),
		trousseau_cli.ShowCommand(),
		trousseau_cli.MetaCommand(),
	}
	app.Flags = []cli.Flag{
		trousseau_cli.VerboseFlag(),
		trousseau_cli.StoreFlag(),
	}
	app.Before = trousseau_cli.Before

	trousseau.Logger.Formatter = new(trousseau.RawFormatter)
	app.Run(os.Args)
}
