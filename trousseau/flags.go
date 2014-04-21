package trousseau

import (
	"github.com/codegangsta/cli"
	"os"
	"path/filepath"
)

func VerboseFlag() cli.BoolFlag {
	return cli.BoolFlag{
		Name:  "verbose",
		Usage: "Set trousseau in verbose mode",
	}
}

func AskPassword() cli.BoolFlag {
	return cli.BoolFlag{
		Name:  "ask-password",
		Usage: "Prompt for password",
	}
}

func YesFlag() cli.StringFlag {
	return cli.StringFlag{
		Name:  "yes",
		Value: "",
		Usage: "Whatever the question is, answers yes",
	}
}

func SshPrivateKeyPathFlag() cli.StringFlag {
	return cli.StringFlag{
		Name:  "ssh-private-key",
		Value: filepath.Join(os.Getenv("HOME"), ".ssh/id_rsa"),
		Usage: "Path to the ssh private key to be used",
	}
}

func OverwriteFlag() cli.BoolFlag {
	return cli.BoolFlag{
		Name:  "overwrite",
		Usage: "Overwrite existing trousseau file",
	}
}

func TheirsFlag() cli.BoolFlag {
	return cli.BoolFlag{
		Name:  "theirs",
		Usage: "Keep the imported file value",
	}
}

func YoursFlag() cli.BoolFlag {
	return cli.BoolFlag{
		Name:  "yours",
		Usage: "Keep your current data store values",
	}
}

func FileFlag() cli.StringFlag {
	return cli.StringFlag{
		Name:  "file",
		Usage: "Path to the file to be extracted",
	}
}
