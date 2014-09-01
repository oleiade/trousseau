package main

import (
	libcli "github.com/codegangsta/cli"
)

func VerboseFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "verbose",
		Usage: "Set trousseau in verbose mode",
	}
}

func NoBackupFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "no-backup",
		Usage: "Don't backup store in the process of upgrading it",
	}
}
