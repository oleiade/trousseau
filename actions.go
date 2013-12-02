package trousseau

import (
	"fmt"
	"github.com/codegangsta/cli"
	"github.com/oleiade/trousseau/dsn"
	"io"
	"log"
	"os"
	"strings"
	"time"
)

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

func CreateAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'configure' command")
	}

	recipients := strings.Split(c.Args()[0], ",")

	meta := Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: TROUSSEAU_VERSION,
	}

	// Create and write empty store file
	CreateStoreFile(gStorePath, &meta)

	fmt.Println("trousseau created")
}

func PushAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'push' command")
	}

	endpointDsn, err := dsn.Parse(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(gS3Defaults)
		if err != nil {
			log.Fatal(err)
		}

		err = uploadUsingS3(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
	case "scp":
		privateKey := c.String("ssh-private-key")

		err := endpointDsn.SetDefaults(gScpDefaults)
		if err != nil {
			log.Fatal(err)
		}

		err = uploadUsingScp(endpointDsn, privateKey)
		if err != nil {
			log.Fatal(err)
		}
	}
}

func PullAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'pull' command")
	}

	endpointDsn, err := dsn.Parse(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(gS3Defaults)
		if err != nil {
			log.Fatal(err)
		}

		err = DownloadUsingS3(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
	case "scp":
		privateKey := c.String("ssh-private-key")

		err := endpointDsn.SetDefaults(gScpDefaults)
		if err != nil {
			log.Fatal(err)
		}

		err = DownloadUsingScp(endpointDsn, privateKey)
		if err != nil {
			log.Fatal(err)
		}
	}
}

func ExportAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'export' command")
	}

	var err error
	var inputFilePath string = gStorePath
	var outputFilePath string = c.Args()[0]

	inputFile, err := os.Open(inputFilePath)
	defer inputFile.Close()
	if err != nil {
		log.Fatal(err)
	}

	outputFile, err := os.Create(outputFilePath)
	defer outputFile.Close()
	if err != nil {
		log.Fatal(err)
	}

	_, err = io.Copy(outputFile, inputFile)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("trousseau exported")
}

func ImportAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'import' command")
	}

	var err error
	var inputFilePath string = c.Args()[0]
	var outputFilePath string = gStorePath
	var inputFile *os.File
	var outputFile *os.File

	inputFile, err = os.Open(inputFilePath)
	defer inputFile.Close()
	if err != nil {
		log.Fatal(err)
	}

	// If trousseau data store does not exist create
	// it through the import process, otherwise just
	// override it's content
	if !pathExists(outputFilePath) {
		outputFile, err = os.Create(outputFilePath)
		defer outputFile.Close()
		if err != nil {
			log.Fatal(err)
		}
	} else {
		outputFile, err = os.OpenFile(outputFilePath, os.O_WRONLY, 0744)
		defer outputFile.Close()
		if err != nil {
			log.Fatal(err)
		}
	}

	_, err = io.Copy(outputFile, inputFile)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("trousseau imported")
}

func AddRecipientAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'add-recipient' command")
	}

	recipient := c.Args()[0]

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	err = store.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.DataStore.Meta.AddRecipient(recipient)

	err = store.Encrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.WriteToFile(gStorePath)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("%s added to trousseau recipients", recipient)
}

func RemoveRecipientAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'remove-recipient' command")
	}

	recipient := c.Args()[0]

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	err = store.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.DataStore.Meta.RemoveRecipient(recipient)

	err = store.Encrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.WriteToFile(gStorePath)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("%s removed from trousseau recipients", recipient)
}

func GetAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'get' command")
	}

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	value, err := store.Get(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("%s key's value: %s\n", c.Args()[0], value)
}

func SetAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 2) {
		log.Fatal("Incorrect number of arguments to 'set' command")
	}

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	err = store.Set(c.Args()[0], c.Args()[1])
	if err != nil {
		log.Fatal(err)
	}

	err = store.WriteToFile(gStorePath)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("key-value pair set: %s:%s\n", c.Args()[0], c.Args()[1])
}

func DelAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'del' command")
	}

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	err = store.Del(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	err = store.WriteToFile(gStorePath)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("%s key deleted\n", c.Args()[0])
}

func KeysAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'keys' command")
	}

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	keys, err := store.Keys()
	if err != nil {
		log.Fatal(err)
	} else {
		for _, k := range keys {
			fmt.Println(k)
		}
	}
}

func ShowAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'show' command")
	}

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	pairs, err := store.Items()
	if err != nil {
		log.Fatal(err)
	} else {
		for _, pair := range pairs {
			fmt.Printf("%s: %s\n", pair.Key, pair.Value)
		}
	}
}

func MetaAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'meta' command")
	}

	store, err := NewEncryptedStoreFromFile(gStorePath, c.GlobalString("passphrase"))
	if err != nil {
		log.Fatal(err)
	}

	pairs, err := store.Meta()
	if err != nil {
		log.Fatal(err)
	}

	for _, pair := range pairs {
		fmt.Printf("%s: %s\n", pair.Key, pair.Value)
	}
}
