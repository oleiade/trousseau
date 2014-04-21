package trousseau

import (
	"fmt"
	"github.com/codegangsta/cli"
	"github.com/oleiade/trousseau/crypto"
	"github.com/oleiade/trousseau/dsn"
	"io"
	"io/ioutil"
	"log"
	"os"
	"strings"
	"time"
)

func CreateAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'configure' command")
	}

	recipients := strings.Split(c.Args()[0], ",")

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
		Recipients: recipients,
	}

	meta := Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: TROUSSEAU_VERSION,
	}

	// Create and write empty store file
	err := CreateStoreFile(gStorePath, opts, &meta)
	if err != nil {
		log.Fatal(err)
	}

	Logger.Info("Trousseau data store succesfully created")
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
		Logger.Info("Trousseau data store succesfully pushed to s3")
	case "scp":
		privateKey := c.String("ssh-private-key")

		err := endpointDsn.SetDefaults(gScpDefaults)
		if err != nil {
			log.Fatal(err)
		}

		if c.Bool("ask-password") == true {
			password := PromptForPassword()
			endpointDsn.Secret = password
		}

		err = uploadUsingScp(endpointDsn, privateKey)
		if err != nil {
			log.Fatal(err)
		}
		Logger.Info("Trousseau data store succesfully pushed to ssh remote storage")
	case "gist":
		err = uploadUsingGist(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
		Logger.Info("Trousseau data store succesfully pushed to gist")
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
		Logger.Info("Trousseau data store succesfully pulled from S3")
	case "scp":
		privateKey := c.String("ssh-private-key")

		err := endpointDsn.SetDefaults(gScpDefaults)
		if err != nil {
			log.Fatal(err)
		}

		if c.Bool("ask-password") == true {
			password := PromptForPassword()
			endpointDsn.Secret = password
		}

		err = DownloadUsingScp(endpointDsn, privateKey)
		if err != nil {
			log.Fatal(err)
		}
		Logger.Info("Trousseau data store succesfully pulled from ssh remote storage")
	case "gist":
		err = DownloadUsingGist(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
		Logger.Info("Trousseau data store succesfully pulled from gist")
	default:
		if endpointDsn.Scheme == "" {
			log.Fatalf("No dsn scheme supplied")
		} else {
			log.Fatalf("Invalid dsn scheme supplied: %s", endpointDsn.Scheme)
		}
	}

	Logger.Info("Trousseau data store succesfully pulled from remote storage")
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

	Logger.Info(fmt.Sprintf("Trousseau data store exported to: %s", outputFilePath))
}

func ImportAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'import' command")
	}

	var err error
	var importedFilePath string = c.Args()[0]
	var localFilePath string = gStorePath
	var strategy *ImportStrategy = new(ImportStrategy)

	// Transform provided merging startegy flags
	// into a proper ImportStrategy byte.
	err = strategy.FromCliContext(c)
	if err != nil {
		log.Fatal(err)
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	localStore, err := LoadStore(localFilePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	importedStore, err := LoadStore(importedFilePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	err = ImportStore(importedStore, localStore, *strategy)
	if err != nil {
		log.Fatal(err)
	}

	err = localStore.Sync()
	if err != nil {
		log.Fatal(err)
	}

	Logger.Info(fmt.Sprintf("Trousseau data store imported: %s", importedFilePath))
}

func AddRecipientAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'add-recipient' command")
	}

	recipient := c.Args()[0]

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	err = store.DataStore.Meta.AddRecipient(recipient)

	err = store.Sync()
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		Logger.Info(fmt.Sprintf("Recipient added to trousseau data store: %s", recipient))
	}
}

func RemoveRecipientAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'remove-recipient' command")
	}

	recipient := c.Args()[0]

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	err = store.Meta.RemoveRecipient(recipient)
	if err != nil {
		log.Fatal(err)
	}

	err = store.Sync()
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		fmt.Printf("Recipient removed from trousseau data store: %s", recipient)
	}
}

func GetAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'get' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	value, err := store.Get(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	// If the --file flag is provided
	if c.String("file") != "" {
		valueBytes, ok := value.(string)
		if !ok {
			log.Fatal(fmt.Sprintf("unable to write %s value to file", c.Args()[0]))
		}

		err := ioutil.WriteFile(c.String("file"), []byte(valueBytes), 0644)
		if err != nil {
			log.Fatal(err)
		}
	} else {
		Logger.Info(value)
	}
}

func SetAction(c *cli.Context) {
	var key string
	var value string
	var err error

	// If the --file flag is provided
	if c.String("file") != "" && hasExpectedArgs(c.Args(), 1) {
		// And the file actually exists on file system
		if pathExists(c.String("file")) {
			// Then load it's content
			fileContent, err := ioutil.ReadFile(c.String("file"))
			if err != nil {
				log.Fatal(err)
			}

			value = string(fileContent)
		} else {
			log.Fatalf("Cannot open %s because it doesn't exist", c.String("file"))
		}
	} else if c.String("file") == "" && hasExpectedArgs(c.Args(), 2) {
		value = c.Args()[1]
	} else {
		log.Fatal("Incorrect number of arguments to 'set' command")
	}

	key = c.Args()[0]

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	err = store.Set(key, value)
	if err != nil {
		log.Fatal(err)
	}

	err = store.Sync()
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		Logger.Info(fmt.Sprintf("%s:%s", key, value))
	}
}

func DelAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'del' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	err = store.Del(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	err = store.Sync()
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		Logger.Info(fmt.Sprintf("deleted: %s", c.Args()[0]))
	}
}

func KeysAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'keys' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	keys, err := store.Keys()
	if err != nil {
		log.Fatal(err)
	} else {
		for _, k := range keys {
			Logger.Info(k)
		}
	}
}

func ShowAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'show' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	pairs, err := store.Items()
	if err != nil {
		log.Fatal(err)
	} else {
		for _, pair := range pairs {
			Logger.Info(fmt.Sprintf("%s : %s", pair.Key, pair.Value))
		}
	}
}

func MetaAction(c *cli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'meta' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: gPasshphrase,
	}

	store, err := LoadStore(gStorePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	pairs, err := store.Metadata()
	if err != nil {
		log.Fatal(err)
	}

	for _, pair := range pairs {
		Logger.Info(fmt.Sprintf("%s : %s", pair.Key, pair.Value))
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
