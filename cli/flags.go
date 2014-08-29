package cli

import (
	libcli "github.com/codegangsta/cli"
	"os"
	"path/filepath"
)

func VerboseFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "verbose",
		Usage: "Set trousseau in verbose mode",
	}
}

func AskPassword() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "ask-password",
		Usage: "Prompt for password",
	}
}

func YesFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "yes",
		Usage: "Whatever the question is, answers yes",
	}
}

func SshPrivateKeyPathFlag() libcli.StringFlag {
	return libcli.StringFlag{
		Name:  "ssh-private-key",
		Value: filepath.Join(os.Getenv("HOME"), ".ssh/id_rsa"),
		Usage: "Path to the ssh private key to be used",
	}
}

func TheirsFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "theirs",
		Usage: "Keep the imported file value",
	}
}

func YoursFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "yours",
		Usage: "Keep your current data store values",
	}
}

func FileFlag() libcli.StringFlag {
	return libcli.StringFlag{
		Name:  "file",
		Usage: "Path to the file to be extracted",
	}
}

func StoreFlag() libcli.StringFlag {
	return libcli.StringFlag{
		Name:  "store",
		Usage: "Path to the trousseau data store to use",
	}
}

func OverwriteFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "overwrite",
		Usage: "Override existing resources",
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
