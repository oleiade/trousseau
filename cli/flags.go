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

func YesFlag() libcli.StringFlag {
	return libcli.StringFlag{
		Name:  "yes",
		Value: "",
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

func OverwriteFlag() libcli.BoolFlag {
	return libcli.BoolFlag{
		Name:  "overwrite",
		Usage: "Overwrite existing trousseau file",
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
