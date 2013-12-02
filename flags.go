package trousseau

import (
	"github.com/codegangsta/cli"
	"os"
	"path/filepath"
)

func PasswordFlag() cli.StringFlag {
	return cli.StringFlag{
        Name: "passphrase",
        Value: GetPassphrase(),
        Usage: "primary gpg key password to decrypt trousseau",
	}
}

func OverwriteFlag() cli.StringFlag {
	return cli.StringFlag{
        Name: "overwrite",
        Value: "",
        Usage: "Overwrite existing trousseau file",
	}
}

func YesFlag() cli.StringFlag {
	return cli.StringFlag{
        Name: "yes",
        Value: "",
        Usage: "Whatever the question is, answers yes",
	}
}

func SshPrivateKeyPathFlag() cli.StringFlag {
	return cli.StringFlag{
        Name: "ssh-private-key",
        Value: filepath.Join(os.Getenv("HOME"), ".ssh/id_rsa"),
        Usage: "Path to the ssh private key to be used",
	}
}
