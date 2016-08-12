package main

import (
	"github.com/oleiade/trousseau"
	"github.com/urfave/cli"
)

func Before(c *cli.Context) error {
	checkHelp(c)
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
