package cli

import (
	"github.com/oleiade/trousseau/trousseau"
	libcli "github.com/codegangsta/cli"
)

func UpdateStorePath(c *libcli.Context) error {
	if c.String("store") != "" {
		trousseau.SetStorePath(c.String("store"))
	}

	return nil
}
