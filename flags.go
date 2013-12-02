package trousseau

import (
	"github.com/codegangsta/cli"
	"os"
	"path/filepath"
)

func PasswordFlag() cli.StringFlag {
	return cli.StringFlag{
		"passphrase",
		GetPassphrase(),
		"primary gpg key password to decrypt trousseau",
	}
}

func OverwriteFlag() cli.StringFlag {
	return cli.StringFlag{
		"overwrite",
		"",
		"Overwrite existing trousseau file",
	}
}

func YesFlag() cli.StringFlag {
	return cli.StringFlag{
		"yes",
		"",
		"Whatever the question is, answers yes",
	}
}

func SshPrivateKeyPathFlag() cli.StringFlag {
	return cli.StringFlag{
		"ssh-private-key",
		filepath.Join(os.Getenv("HOME"), ".ssh/id_rsa"),
		"Path to the ssh private key to be used",
	}
}
