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

func StoreFlag() libcli.StringFlag {
	return libcli.StringFlag{
		Name:  "store, s",
		Usage: "Path to the trousseau data store to use",
	}
}

func PlainFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "plain",
		Usage: "Import or export plain",
	}
}

func NoBackupFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "no-backup",
		Usage: "Don't backup store in the process of upgrading it",
	}
}
