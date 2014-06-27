package cli

import (
	"fmt"
	libcli "github.com/codegangsta/cli"
	"github.com/oleiade/trousseau/crypto"
	"github.com/oleiade/trousseau/dsn"
	"github.com/oleiade/trousseau/trousseau"
	"io"
	"io/ioutil"
	"log"
	"os"
	"strings"
	"time"
)

func CreateAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'configure' command")
	}

	recipients := strings.Split(c.Args()[0], ",")

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
		Recipients: recipients,
	}

	meta := trousseau.Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: trousseau.TROUSSEAU_VERSION,
	}

	// Create and write empty store file
	err := trousseau.CreateStoreFile(trousseau.GetStorePath(), opts, &meta)
	if err != nil {
		log.Fatal(err)
	}

	trousseau.Logger.Info("Trousseau data store succesfully created")
}

func PushAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'push' command")
	}

	endpointDsn, err := dsn.Parse(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(trousseau.S3Defaults)
		if err != nil {
			log.Fatal(err)
		}

		err = trousseau.UploadUsingS3(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
		trousseau.Logger.Info("Trousseau data store succesfully pushed to s3")
	case "scp":
		privateKey := c.String("ssh-private-key")

		err := endpointDsn.SetDefaults(trousseau.ScpDefaults)
		if err != nil {
			log.Fatal(err)
		}

		if c.Bool("ask-password") == true {
			password := trousseau.PromptForPassword()
			endpointDsn.Secret = password
		}

		err = trousseau.UploadUsingScp(endpointDsn, privateKey)
		if err != nil {
			log.Fatal(err)
		}
		trousseau.Logger.Info("Trousseau data store succesfully pushed to ssh remote storage")
	case "gist":
		err = trousseau.UploadUsingGist(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
		trousseau.Logger.Info("Trousseau data store succesfully pushed to gist")
	}
}

func PullAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'pull' command")
	}

	endpointDsn, err := dsn.Parse(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(trousseau.S3Defaults)
		if err != nil {
			log.Fatal(err)
		}

		err = trousseau.DownloadUsingS3(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
		trousseau.Logger.Info("Trousseau data store succesfully pulled from S3")
	case "scp":
		privateKey := c.String("ssh-private-key")

		err := endpointDsn.SetDefaults(trousseau.ScpDefaults)
		if err != nil {
			log.Fatal(err)
		}

		if c.Bool("ask-password") == true {
			password := trousseau.PromptForPassword()
			endpointDsn.Secret = password
		}

		err = trousseau.DownloadUsingScp(endpointDsn, privateKey)
		if err != nil {
			log.Fatal(err)
		}
		trousseau.Logger.Info("Trousseau data store succesfully pulled from ssh remote storage")
	case "gist":
		err = trousseau.DownloadUsingGist(endpointDsn)
		if err != nil {
			log.Fatal(err)
		}
		trousseau.Logger.Info("Trousseau data store succesfully pulled from gist")
	default:
		if endpointDsn.Scheme == "" {
			log.Fatalf("No dsn scheme supplied")
		} else {
			log.Fatalf("Invalid dsn scheme supplied: %s", endpointDsn.Scheme)
		}
	}

	trousseau.Logger.Info("Trousseau data store succesfully pulled from remote storage")
}

func ExportAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'export' command")
	}

	var err error
	var inputFilePath string = trousseau.GetStorePath()
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

	trousseau.Logger.Info(fmt.Sprintf("Trousseau data store exported to: %s", outputFilePath))
}

func ImportAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'import' command")
	}

	var err error
	var importedFilePath string = c.Args()[0]
	var localFilePath string = trousseau.GetStorePath()
	var strategy *trousseau.ImportStrategy = new(trousseau.ImportStrategy)

	// Transform provided merging startegy flags
	// into a proper ImportStrategy byte.
	err = strategy.FromCliContext(c)
	if err != nil {
		log.Fatal(err)
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	localStore, err := trousseau.LoadStore(localFilePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	importedStore, err := trousseau.LoadStore(importedFilePath, opts)
	if err != nil {
		log.Fatal(err)
	}

	err = trousseau.ImportStore(importedStore, localStore, *strategy)
	if err != nil {
		log.Fatal(err)
	}

	err = localStore.Sync()
	if err != nil {
		log.Fatal(err)
	}

	trousseau.Logger.Info(fmt.Sprintf("Trousseau data store imported: %s", importedFilePath))
}

func AddRecipientAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'add-recipient' command")
	}

	recipient := c.Args()[0]

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
	if err != nil {
		log.Fatal(err)
	}

	err = store.DataStore.Meta.AddRecipient(recipient)

	err = store.Sync()
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		trousseau.Logger.Info(fmt.Sprintf("Recipient added to trousseau data store: %s", recipient))
	}
}

func RemoveRecipientAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'remove-recipient' command")
	}

	recipient := c.Args()[0]

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
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

func GetAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'get' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
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
		// Use fmt.Print to support patch processes,
		// to get the value "as is" without any appended newlines
		if isPipe(os.Stdout) {
			fmt.Print(value)
		} else {
			trousseau.Logger.Info(value)
		}
	}
}

func SetAction(c *libcli.Context) {
	var key string
	var value string
	var err error

	// If the --file flag is provided
	if c.String("file") != "" && hasExpectedArgs(c.Args(), 1) {
		// And the file actually exists on file system
		if trousseau.PathExists(c.String("file")) {
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
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
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
		trousseau.Logger.Info(fmt.Sprintf("%s:%s", key, value))
	}
}

func DelAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'del' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
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
		trousseau.Logger.Info(fmt.Sprintf("deleted: %s", c.Args()[0]))
	}
}

func KeysAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'keys' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
	if err != nil {
		log.Fatal(err)
	}

	keys, err := store.Keys()
	if err != nil {
		log.Fatal(err)
	} else {
		for _, k := range keys {
			trousseau.Logger.Info(k)
		}
	}
}

func ShowAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'show' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
	if err != nil {
		log.Fatal(err)
	}

	pairs, err := store.Items()
	if err != nil {
		log.Fatal(err)
	} else {
		for _, pair := range pairs {
			trousseau.Logger.Info(fmt.Sprintf("%s : %s", pair.Key, pair.Value))
		}
	}
}

func MetaAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'meta' command")
	}

	opts := &crypto.Options{
		Algorithm:  crypto.GPG_ENCRYPTION,
		Passphrase: trousseau.GetPassphrase(),
	}

	store, err := trousseau.LoadStore(trousseau.GetStorePath(), opts)
	if err != nil {
		log.Fatal(err)
	}

	pairs, err := store.Metadata()
	if err != nil {
		log.Fatal(err)
	}

	for _, pair := range pairs {
		trousseau.Logger.Info(fmt.Sprintf("%s : %s", pair.Key, pair.Value))
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

// Thanks, mrnugget!
// https://github.com/mrnugget/fzz/blob/master/utils.go#L14-L21
func isPipe(f *os.File) bool {
	s, err := f.Stat()
	if err != nil {
		return false
	}

	return s.Mode()&os.ModeNamedPipe != 0
}
