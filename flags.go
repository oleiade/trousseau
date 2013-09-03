package trousseau

import (
	"github.com/codegangsta/cli"
)

func PasswordFlag() cli.StringFlag {
	return cli.StringFlag{
		"password",
		"",
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

func RemoteStorageFlag() cli.StringFlag {
	return cli.StringFlag{
		"remote-storage",
		"s3",
		"Remote storage type to use: s3 or scp",
	}
}

func RemoteFilenameFlag() cli.StringFlag {
	return cli.StringFlag{
		"remote-filename",
		"",
		"Remote name of the trousseau file",
	}
}

func RemoteHostFlag() cli.StringFlag {
	return cli.StringFlag{
		"host",
		"",
		"Remote storage hostname",
	}
}

func RemotePortFlag() cli.StringFlag {
	return cli.StringFlag{
		"port",
		"22",
		"Port to be used for remote storage connexion",
	}
}

func RemoteUserFlag() cli.StringFlag {
	return cli.StringFlag{
		"user",
		"",
		"User to be used for remote storage connexion",
	}
}

func S3BucketFlag() cli.StringFlag {
	return cli.StringFlag{
		"s3-bucket",
		"",
		"S3 name of the bucket hosting the trousseau file",
	}
}

func SshPrivateKeyPathFlag() cli.StringFlag {
	return cli.StringFlag{
		"ssh-private-key",
		"",
		"Path to the ssh private key to be used",
	}
}
