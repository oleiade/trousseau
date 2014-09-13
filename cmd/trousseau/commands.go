package main

import (
	"github.com/codegangsta/cli"
	"github.com/oleiade/trousseau"
	"fmt"
	"os"
	"strings"
	"path/filepath"
)

func CreateCommand() cli.Command {
	return cli.Command{
		Name:   "create",
		Usage:  "Create an encrypted data store",
		Description: "The create command will generate an encrypted data store " +
					 "placed at $HOME/.trousseau.tr or at the location described by " +
					 "the $TROUSSEAU_HOME environment variable if you provided it.\n\n" +
					 "   Encryption is made using your GPG main identity, and targets the " +
					 "GPG recipients you provide as the command arguments.\n\n" +
					 "   Examples:\n\n" +
					 "     trousseau create 16DB4F3\n" +
					 "     trousseau create tcrevon@gmail.com\n" +
					 "     export TROUSSEAU_STORE=/tmp/test_trousseau.tr && trousseau create 16DB4F3\n",
		Action: func(c *cli.Context) {
			var recipients []string = strings.Split(c.Args()[0], ",")
			trousseau.CreateAction(recipients)
		},
	}
}

func PushCommand() cli.Command {
	return cli.Command{
		Name:   "push",
		Usage:  "Push the encrypted data store to a remote storage",
		Description: "The local encrypted data store will be pushed to a remote destination " +
					 "described by a data source name.\n\n" +
					 "   Trousseau data source name goes as follow:\n\n" +
					 "     {protocol}://{identifier}:{secret}@{host}:{port}/{path}\n\n" +
					 "   Given:\n" +
					 "     * protocol: The remote service target type. Can be one of: s3 or scp\n" +
					 "     * identifier: The login/key/whatever to authenticate trousseau to the remote service. Provide your aws_access_key if you're targeting s3, or your remote login if you're targeting scp\n" +
					 "     * secret: The secret to authenticate trousseau to the remote service. Provide your aws_secret_key if you're targeting s3, or your remote password if you're targeting scp\n" +
					 "     * host: Your bucket name is you're targeting s3. The host to login to using scp otherwise\n" +
					 "     * port: The aws_region if you're targeting s3. The port to login to using scp otherwise\n" +
					 "     * path: The remote path to push to or retrieve from the trousseau file on a push or pull action\n\n" +
					 "   Examples:\n\n" +
					 "     s3://1298u1928eu9182dj19d2:1928u192ijdnh1b2d8@my-super-bucket:eu-west-1/topsecret-trousseau.tr\n" +
					 "     scp://myuser:@myhost.io:6453/topsecret-trousseau.tr  (use the password option to supply password)\n" +
					 "     gist://oleiade:1928u3019j2d9812dn0192u490128dj@:/topsecret-trousseau.tr\n",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to push command")
			}

			var destination string = c.Args().First()
			trousseau.PushAction(destination, c.String("ssh-private-key"), c.Bool("ask-password"))
		},
		Flags: []cli.Flag{
			cli.BoolFlag{
				Name:  "overwrite",
				Usage: "Overwrite any existing remote resource with pushed data",
			},
			cli.BoolFlag{
				Name:  "password",
				Usage: "Prompt for remote host ssh password",
			},
			cli.StringFlag{
				Name:  "ssh-private-key",
				Value: filepath.Join(os.Getenv("HOME"), ".ssh/id_rsa"),
				Usage: "Path to the ssh private key to be used when pushing to remote storage via ssh",
			},
		},
	}
}

func PullCommand() cli.Command {
	return cli.Command{
		Name:   "pull",
		Usage:  "Pull the encrypted data store from a remote storage",
		Description: "The remote encrypted data store described by a data source name " +
					 "will be pulled and replace the local data store.\n\n" +
					 "   Trousseau data source name goes as follow:\n\n" +
					 "     {protocol}://{identifier}:{secret}@{host}:{port}/{path}\n\n" +
					 "   Given:\n" +
					 "     * protocol: The remote service target type. Can be one of: s3 or scp\n" +
					 "     * identifier: The login/key/whatever to authenticate trousseau to the remote service. Provide your aws_access_key if you're targeting s3, or your remote login if you're targeting scp\n" +
					 "     * secret: The secret to authenticate trousseau to the remote service. Provide your aws_secret_key if you're targeting s3, or your remote password if you're targeting scp\n" +
					 "     * host: Your bucket name is you're targeting s3. The host to login to using scp otherwise\n" +
					 "     * port: The aws_region if you're targeting s3. The port to login to using scp otherwise\n" +
					 "     * path: The remote path to push to or retrieve from the trousseau file on a push or pull action\n\n" +
					 "   Examples:\n\n" +
					 "     s3://1298u1928eu9182dj19d2:1928u192ijdnh1b2d8@my-super-bucket:eu-west-1/topsecret-trousseau.tr\n" +
					 "     scp://myuser:@myhost.io:6453/topsecret-trousseau.tr  (use the password option to supply password)\n" +
					 "     gist://oleiade:1928u3019j2d9812dn0192u490128dj@:/topsecret-trousseau.tr\n",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to pull command")
			}

			var source string = c.Args().First()
			trousseau.PullAction(source, c.String("ssh-private-key"), c.Bool("ask-password"))
		},
		Flags: []cli.Flag{
			cli.BoolFlag{
				Name:  "overwrite",
				Usage: "Overwrite local data store with pulled remote resource",
			},
			cli.BoolFlag{
				Name:  "password",
				Usage: "Prompt for remote host ssh password",
			},
			cli.StringFlag{
				Name:  "ssh-private-key",
				Value: filepath.Join(os.Getenv("HOME"), ".ssh/id_rsa"),
				Usage: "Path to the ssh private key to be used when pulling from remote storage via ssh",
			},
		},
	}
}

func ExportCommand() cli.Command {
	return cli.Command{
		Name:   "export",
		Usage:  "Export the encrypted data store to a file system location",
		Description: "The encrypted data store at the default location ($HOME/.trousseau.tr) or " +
					 "the one pointed by the $TROUSSEAU_STORE environment variable will be pushed as is " +
					 "to the filesystem location provided as first argument.",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to export command")
			}

			var to string = c.Args().First()
			trousseau.ExportAction(to, c.Bool("plain"))
		},
		Flags: []cli.Flag{
			cli.BoolFlag{
				Name:  "overwrite",
				Usage: "Overwrite any existing destination resource",
			},
			cli.BoolFlag{
				Name:  "plain",
				Usage: "Export the plain content of the encrypted data store",
			},
		},
	}
}

func ImportCommand() cli.Command {
	return cli.Command{
		Name:   "import",
		Usage:  "Import an encrypted data store from a file system location",
		Description: "The encrypted data store at the filesystem location provided as first argument " +
					 "will be imported to the default trousseau location ($HOME/.trousseau.tr) or " +
					 "the one pointed by the $TROUSSEAU_STORE environment variable",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to import command")
			}

			var strategy trousseau.ImportStrategy
			var yours bool = c.Bool("yours")
			var theirs bool = c.Bool("theirs")
			var overwrite bool = c.Bool("overwrite")
			var activated uint = 0

			// Ensure two import strategies were not provided at
			// the same time. Otherwise, throw an error
			for _, flag := range []bool{yours, theirs, overwrite} {
				if flag {
					activated += 1
				}
				if activated >= 2 {
					trousseau.ErrorLogger.Fatal("--yours, --theirs and --overwrite options are mutually exclusive")
				}
			}

			// Return proper ImportStrategy according to
			// provided flags
			if overwrite == true {
				strategy = trousseau.IMPORT_OVERWRITE
			} else if theirs == true {
				strategy = trousseau.IMPORT_THEIRS
			} else {
				strategy = trousseau.IMPORT_YOURS
			}

			var from string = c.Args().First()
			trousseau.ImportAction(from, strategy, c.Bool("plain"))
		},
		Flags: []cli.Flag{
			cli.BoolFlag{
				Name:  "overwrite",
				Usage: "Overwrite local data store with imported resource",
			},
			cli.BoolFlag{
				Name:  "plain",
				Usage: "Import the content of the encrypted data store from a plain file",
			},
			cli.BoolFlag{
				Name:  "theirs",
				Usage: "Keep the imported file value",
			},
			cli.BoolFlag{
				Name:  "yours",
				Usage: "Keep your current data store values",
			},
		},
	}
}

func ListRecipientsCommand() cli.Command {
	return cli.Command{
		Name:   "list-recipients",
		Usage:  "List the data store encryption recipients",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to list-recipients command")
			}

			trousseau.ListRecipientsAction()
		},
	}
}

func AddRecipientCommand() cli.Command {
	return cli.Command{
		Name:   "add-recipient",
		Usage:  "Add a recipient to the encrypted data store",
		Description: "Add a valid GPG recipient to the encrypted data store. To proceed you must " +
					 "make sure the recipient's GPG public key is available in your public keyring (this " +
					 "can be done by making sure it appears in the 'gpg --list-keys' command's output).\n" +
					 "   And you can whether provide it whether as an openpgp id or by using the email attached " +
					 "to it's key",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to add-recipient command")
			}

			trousseau.AddRecipientAction(c.Args().First())

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("Recipient added to trousseau data store: %s", c.Args().First()))
			}
		},
	}
}

func RemoveRecipientCommand() cli.Command {
	return cli.Command{
		Name:   "remove-recipient",
		Usage:  "Remove a recipient from the encrypted data store",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to remove-recipient command")
			}

			trousseau.RemoveRecipientAction(c.Args().First())

			if c.Bool("verbose") == true {
				fmt.Printf("Recipient removed from trousseau data store: %s", c.Args().First())
			}

		},
	}
}

func SetCommand() cli.Command {
	return cli.Command{
		Name:   "set",
		Usage:  "Set a key value pair in the encrypted data store",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 2) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to set command")
			}

			var key string = c.Args().First()
			var value string = c.Args()[1]
			var file string = c.String("file")

			trousseau.SetAction(key, value, file)

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("%s:%s", key, value))
			}
		},
		Flags: []cli.Flag{
			cli.StringFlag{
				Name:  "file, f",
				Usage: "Write key's value to provided file",
			},
		},
	}
}

func GetCommand() cli.Command {
	return cli.Command{
		Name:   "get",
		Usage:  "Get a key's value from the encrypted data store",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to get command")
			}

			var key string = c.Args().First()
			var file string = c.String("file")
			trousseau.GetAction(key, file)
		},
		Flags: []cli.Flag{
			cli.StringFlag{
				Name:  "file, f",
				Usage: "Read key's value from provided file",
			},
		},
	}
}

func RenameCommand() cli.Command {
	return cli.Command{
		Name:   "rename",
		Usage:  "Rename an encrypted data store's key",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 2) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to rename command")
			}

			var src string = c.Args().First()
			var dest string = c.Args()[1]

			trousseau.RenameAction(src, dest, c.Bool("overwrite"))

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("renamed: %s to %s", src, dest))
			}
		},
		Flags: []cli.Flag{
			cli.BoolFlag{
				Name:  "overwrite",
				Usage: "Override any existing destination key",
			},
		},
	}
}

func DelCommand() cli.Command {
	return cli.Command{
		Name:   "del",
		Usage:  "Delete a key value pair from the store",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to del command")
			}

			var key string = c.Args().First()

			trousseau.DelAction(key)

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("deleted: %s", c.Args()[0]))
			}
		},
	}
}

func KeysCommand() cli.Command {
	return cli.Command{
		Name:   "keys",
		Usage:  "List the encrypted data store keys",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to keys command")
			}

			trousseau.KeysAction()
		},
	}
}

func ShowCommand() cli.Command {
	return cli.Command{
		Name:   "show",
		Usage:  "Show the encrypted data store key value pairs",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to show command")
			}

			trousseau.ShowAction()
		},
	}
}

func MetaCommand() cli.Command {
	return cli.Command{
		Name:   "meta",
		Usage:  "Show the encrypted data store metadata",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to meta command")
			}

			trousseau.MetaAction()
		},
	}
}

func UpgradeCommand() cli.Command {
	return cli.Command{
		Name:   "upgrade",
		Usage:  "Upgrade the encrypted data store to a newer version's file format",
		Action: func(c *cli.Context) {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to upgrade command")
			}

			trousseau.UpgradeAction(c.Bool("yes"), c.Bool("no-backup"))
		},
		Flags: []cli.Flag{
			cli.BoolFlag{
				Name:  "yes, y",
				Usage: "Answer yes when prompted to trigger the upgrade action",
			},
			cli.BoolFlag{
				Name:  "no-backup",
				Usage: "Don't backup store in the process of upgrading it",
			},
		},
	}
}

// hasExpectedArgs checks whether the number of args are as expected.
func hasExpectedArgs(args []string, expected int) bool {
	switch expected {
	case -1:
		if len(args) > 0 {
			return true
		} else {
			return false
		}
	default:
		if len(args) == expected {
			return true
		} else {
			return false
		}
	}
}

