package main

import (
	"github.com/codegangsta/cli"
	"github.com/oleiade/trousseau"
	"os"
)

func main() {
	app := cli.NewApp()

	app.Name = "trousseau"
	app.Usage = "handles an encrypted keys store"
	app.Version = trousseau.TROUSSEAU_VERSION
	app.Commands = []cli.Command{
		trousseau.CreateCommand(),
		trousseau.PushCommand(),
		trousseau.PullCommand(),
		trousseau.ExportCommand(),
		trousseau.ImportCommand(),
		trousseau.AddRecipientCommand(),
		trousseau.RemoveRecipientCommand(),
		trousseau.SetCommand(),
		trousseau.GetCommand(),
		trousseau.DelCommand(),
		trousseau.KeysCommand(),
		trousseau.ShowCommand(),
		trousseau.MetaCommand(),
	}

    // Application global options
	app.Flags = []cli.Flag{
        trousseau.PasswordFlag(),
	}

	app.Run(os.Args)
}
