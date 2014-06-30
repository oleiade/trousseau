package cli

import (
	libcli "github.com/codegangsta/cli"
	"github.com/oleiade/trousseau/trousseau"
)

func Before(c *libcli.Context) error {
	var err error

	err = checkHelp(c)
	if err != nil {
		return err
	}

	err = updateStorePath(c)
	if err != nil {
		return err
	}

	return nil
}

// checkHelp will print command or app help according to the
// provided context. It is used to bypass the gpg key check
// before the application runs. So users can print the help
// without selecting their master key.
func checkHelp(c *libcli.Context) error {
	if c.GlobalBool("h") || c.GlobalBool("help") {
		if len(c.Args()) >= 1 {
			libcli.ShowCommandHelp(c, c.Args().First())
		} else {
			libcli.ShowAppHelp(c)
		}
	}

	return nil
}

// updateStorePath selects the default trousseau data store if
// none were provided on the command line
func updateStorePath(c *libcli.Context) error {
	if c.String("store") != "" {
		trousseau.SetStorePath(c.String("store"))
	}

	return nil
}
