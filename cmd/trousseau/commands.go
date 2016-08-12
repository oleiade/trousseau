package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"

	"github.com/oleiade/trousseau"
	"github.com/urfave/cli"
)

func CreateCommand() cli.Command {
	return cli.Command{
		Name:  "create",
		Usage: "Create an encrypted data store",
		Description: "The create command will generate an encrypted data store " +
			"placed at $HOME/.trousseau.tr or at the location described by " +
			"the $TROUSSEAU_HOME environment variable if you provided it.\n\n" +
			"   Encryption is made using your GPG main identity, and targets the " +
			"GPG recipients you provide as the command arguments.\n\n" +
			"   Examples:\n\n" +
			"     trousseau create 16DB4F3\n" +
			"     trousseau create tcrevon@gmail.com\n" +
			"     export TROUSSEAU_STORE=/tmp/test_trousseau.tr && trousseau create 16DB4F3\n",
		Action: func(c *cli.Context) error {
			var encryptionType string = c.String("encryption-type")

			if encryptionType == trousseau.SYMMETRIC_ENCRYPTION_REPR {
				err := trousseau.CreateAction(trousseau.SYMMETRIC_ENCRYPTION, trousseau.AES_256_ENCRYPTION, nil)
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}
			} else {
				if len(c.Args()) > 0 {
					err := trousseau.CreateAction(trousseau.ASYMMETRIC_ENCRYPTION, trousseau.GPG_ENCRYPTION, c.Args())
					if err != nil {
						trousseau.ErrorLogger.Fatal(err)
					}
				} else {
					trousseau.ErrorLogger.Fatal("invalid number of arguments provided to " +
						"the create command. At least one recipient to encrypt the " +
						"data store for is needed.")
				}
			}

			trousseau.InfoLogger.Println("Trousseau data store succesfully created")

			return nil
		},
		Flags: []cli.Flag{
			cli.StringFlag{
				Name: "encryption-type",
				Usage: "Define the encryption type to be used for store encryption. " +
					"Whether symmetric or asymmetric.",
				Value: trousseau.ASYMMETRIC_ENCRYPTION_REPR,
			},
			cli.StringFlag{
				Name: "encryption-algorithm",
				Usage: "Define the algorithm to be used for store encryption. " +
					"Whether gpg or aes.",
				Value: trousseau.GPG_ENCRYPTION_REPR,
			},
		},
	}
}

func PushCommand() cli.Command {
	return cli.Command{
		Name:  "push",
		Usage: "Push the encrypted data store to a remote storage",
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
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to push command")
			}

			var destination string = c.Args().First()
			err := trousseau.PushAction(destination, c.String("ssh-private-key"), c.Bool("ask-password"))
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			trousseau.InfoLogger.Printf("Encrypted data store succesfully pushed to %s remote storage\n", destination)

			return nil
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
		Name:  "pull",
		Usage: "Pull the encrypted data store from a remote storage",
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
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to pull command")
			}

			var source string = c.Args().First()

			err := trousseau.PullAction(source, c.String("ssh-private-key"), c.Bool("ask-password"))
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			trousseau.InfoLogger.Println("Encrypted data store succesfully pulled from remote storage\n")

			return nil
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
		Name:  "export",
		Usage: "Export the encrypted data store to a file system location",
		Description: "The encrypted data store at the default location ($HOME/.trousseau.tr) or " +
			"the one pointed by the $TROUSSEAU_STORE environment variable will be pushed as is " +
			"to the filesystem location provided as first argument.",
		Action: func(c *cli.Context) error {
			// TODO: restore with further version of hasExpectedArgs
			//			if !hasExpectedArgs(c.Args(), 1) {
			//				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to export command")
			//			}
			if len(c.Args()) == 0 {
				destination := os.Stdout
				err := trousseau.ExportAction(destination, c.Bool("plain"))
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}
			} else if len(c.Args()) == 1 {
				destination, err := os.Create(c.Args().First())
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}
				defer destination.Close()

				// Make sure the file is readble/writable only
				// by its owner
				err = os.Chmod(destination.Name(), os.FileMode(0600))
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}

				err = trousseau.ExportAction(destination, c.Bool("plain"))
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}

				trousseau.InfoLogger.Printf("Data store exported to: %s", c.Args().First())
			}

			return nil
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
		Name:  "import",
		Usage: "Import an encrypted data store from a file system location",
		Description: "The encrypted data store at the filesystem location provided as first argument " +
			"will be imported to the default trousseau location ($HOME/.trousseau.tr) or " +
			"the one pointed by the $TROUSSEAU_STORE environment variable",
		Action: func(c *cli.Context) error {
			//			if !hasExpectedArgs(c.Args(), 1) {
			//				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to import command")
			//			}
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

			if len(c.Args()) == 0 {
				source := os.Stdin
				err := trousseau.ImportAction(source, strategy, c.Bool("plain"))
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}
			} else if len(c.Args()) == 1 {
				source, err := os.Open(c.Args().First())
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}
				defer source.Close()

				err = trousseau.ImportAction(source, strategy, c.Bool("plain"))
				if err != nil {
					trousseau.ErrorLogger.Fatal(err)
				}

			}

			trousseau.InfoLogger.Println(fmt.Sprintf("Trousseau data store imported: %s", c.Args().First()))

			return nil
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
		Name:  "list-recipients",
		Usage: "List the data store encryption recipients",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to list-recipients command")
			}

			err := trousseau.ListRecipientsAction()
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			return nil
		},
	}
}

func AddRecipientCommand() cli.Command {
	return cli.Command{
		Name:  "add-recipient",
		Usage: "Add a recipient to the encrypted data store",
		Description: "Add a valid GPG recipient to the encrypted data store. To proceed you must " +
			"make sure the recipient's GPG public key is available in your public keyring (this " +
			"can be done by making sure it appears in the 'gpg --list-keys' command's output).\n" +
			"   And you can whether provide it whether as an openpgp id or by using the email attached " +
			"to it's key",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to add-recipient command")
			}

			err := trousseau.AddRecipientAction(c.Args().First())
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("Recipient added to trousseau data store: %s", c.Args().First()))
			}

			return nil
		},
	}
}

func RemoveRecipientCommand() cli.Command {
	return cli.Command{
		Name:  "remove-recipient",
		Usage: "Remove a recipient from the encrypted data store",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to remove-recipient command")
			}

			err := trousseau.RemoveRecipientAction(c.Args().First())
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			if c.Bool("verbose") == true {
				fmt.Printf("Recipient removed from trousseau data store: %s", c.Args().First())
			}

			return nil
		},
	}
}

func SetCommand() cli.Command {
	return cli.Command{
		Name:  "set",
		Usage: "Set a key value pair in the encrypted data store",
		Action: func(c *cli.Context) error {
			var file string = c.String("file")
			var key string = c.Args().First()
			var value string

			if file != "" {
				if !hasExpectedArgs(c.Args(), 1) {
					trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to set command")
				}
			} else {
				if !hasExpectedArgs(c.Args(), 1) && !hasExpectedArgs(c.Args(), 2) {
					trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to set command")
				}

				if hasExpectedArgs(c.Args(), 2) {
					value = c.Args()[1]
				} else if hasExpectedArgs(c.Args(), 1) {
					var err error
					reader := bufio.NewReader(os.Stdin)
					value, err = reader.ReadString('\n')
					if err != nil {
						trousseau.ErrorLogger.Fatal(err)
					}
					value = value[:len(value)-1]
				}
			}

			err := trousseau.SetAction(key, value, file)
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("%s:%s", key, value))
			}

			return nil
		},
		Flags: []cli.Flag{
			cli.StringFlag{
				Name:  "file, f",
				Usage: "Read key's value from provided file",
			},
		},
	}
}

func GetCommand() cli.Command {
	return cli.Command{
		Name:  "get",
		Usage: "Get a key's value from the encrypted data store",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to get command")
			}

			var key string = c.Args().First()
			var file string = c.String("file")

			err := trousseau.GetAction(key, file)
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			return nil
		},
		Flags: []cli.Flag{
			cli.StringFlag{
				Name:  "file, f",
				Usage: "Write key's value to provided file",
			},
		},
	}
}

func RenameCommand() cli.Command {
	return cli.Command{
		Name:  "rename",
		Usage: "Rename an encrypted data store's key",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 2) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to rename command")
			}

			var src string = c.Args().First()
			var dest string = c.Args()[1]

			err := trousseau.RenameAction(src, dest, c.Bool("overwrite"))
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("renamed: %s to %s", src, dest))
			}

			return nil
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
		Name:  "del",
		Usage: "Delete a key value pair from the store",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 1) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to del command")
			}

			var key string = c.Args().First()

			err := trousseau.DelAction(key)
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			if c.Bool("verbose") == true {
				trousseau.InfoLogger.Println(fmt.Sprintf("deleted: %s", c.Args()[0]))
			}

			return nil
		},
	}
}

func KeysCommand() cli.Command {
	return cli.Command{
		Name:  "keys",
		Usage: "List the encrypted data store keys",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to keys command")
			}

			err := trousseau.KeysAction()
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			return nil
		},
	}
}

func ShowCommand() cli.Command {
	return cli.Command{
		Name:  "show",
		Usage: "Show the encrypted data store key value pairs",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to show command")
			}

			err := trousseau.ShowAction()
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			return nil
		},
	}
}

func MetaCommand() cli.Command {
	return cli.Command{
		Name:  "meta",
		Usage: "Show the encrypted data store metadata",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to meta command")
			}

			err := trousseau.MetaAction()
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			return nil
		},
	}
}

func UpgradeCommand() cli.Command {
	return cli.Command{
		Name:  "upgrade",
		Usage: "Upgrade the encrypted data store to a newer version's file format",
		Action: func(c *cli.Context) error {
			if !hasExpectedArgs(c.Args(), 0) {
				trousseau.ErrorLogger.Fatal("Invalid number of arguments provided to upgrade command")
			}

			err := trousseau.UpgradeAction(c.Bool("yes"), c.Bool("no-backup"))
			if err != nil {
				trousseau.ErrorLogger.Fatal(err)
			}

			trousseau.InfoLogger.Print("Data store succesfully upgraded")

			return nil
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
