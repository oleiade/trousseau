package cli

import (
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"os"
	"strings"
	"time"

	libcli "github.com/codegangsta/cli"
	"github.com/oleiade/trousseau/dsn"
	"github.com/oleiade/trousseau/trousseau"
	"encoding/json"
)

func CreateAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'configure' command")
	}

	recipients := strings.Split(c.Args()[0], ",")

	meta := trousseau.Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: trousseau.TROUSSEAU_VERSION,
	}
	store := trousseau.NewStore(meta)

	tr := trousseau.Trousseau{
		CryptoType:      trousseau.ASYMMETRIC_ENCRYPTION,
		CryptoAlgorithm: trousseau.GPG_ENCRYPTION,
	}
	tr.Encrypt(store)

	err := tr.Write(trousseau.InferStorePath())
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
	var outputFilePath string = c.Args()[0]


	outputFile, err := os.Create(outputFilePath)
	defer outputFile.Close()
	if err != nil {
		log.Fatal(err)
	}

	// Make sure the file is readble/writable only
	// by its owner
	err = os.Chmod(outputFile.Name(), os.FileMode(0600))
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("plain") == true {
		tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
		if err != nil {
			log.Fatal(err)
		}

		store, err := tr.Decrypt()
		if err != nil {
			log.Fatal(err)
		}

		storeBytes, err := json.Marshal(store)
		if err != nil {
			log.Fatal(err)
		}

		err = ioutil.WriteFile(outputFilePath, storeBytes, os.FileMode(0600))
		if err != nil {
			log.Fatal(err)
		}
	} else {
		inputFile, err := os.Open(trousseau.InferStorePath())
		defer inputFile.Close()
		if err != nil {
			log.Fatal(err)
		}

		_, err = io.Copy(outputFile, inputFile)
		if err != nil {
			log.Fatal(err)
		}
	}

	trousseau.Logger.Info(fmt.Sprintf("Trousseau data store exported to: %s", outputFilePath))
}

func ImportAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'import' command")
	}

	var err error
	var importedFilePath string = c.Args()[0]
	var importedStore *trousseau.Store = &trousseau.Store{}
	var localFilePath string = trousseau.InferStorePath()
	var strategy *trousseau.ImportStrategy = new(trousseau.ImportStrategy)

	// Transform provided merging startegy flags
	// into a proper ImportStrategy byte.
	err = strategy.FromCliContext(c)
	if err != nil {
		log.Fatal(err)
	}

	localTr, err := trousseau.OpenTrousseau(localFilePath)
	if err != nil {
		log.Fatal(err)
	}

	localStore, err := localTr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}


	if c.Bool("plain") == true {
		importedData, err := ioutil.ReadFile(importedFilePath)
		if err != nil {
			log.Fatal(err)
		}

		err = json.Unmarshal(importedData, importedStore)
		if err != nil {
			log.Fatal(err)
		}
	} else {
		importedTr, err := trousseau.OpenTrousseau(importedFilePath)
		if err != nil {
			log.Fatal(err)
		}

		importedStore, err = importedTr.Decrypt()
		if err != nil {
			log.Fatal(err)
		}
	}


	err = trousseau.ImportStore(importedStore, localStore, *strategy)
	if err != nil {
		log.Fatal(err)
	}

	err = localTr.Encrypt(localStore)
	if err != nil {
		log.Fatal(err)
	}

	err = localTr.Write(localFilePath)
	if err != nil {
		log.Fatal(err)
	}

	trousseau.Logger.Info(fmt.Sprintf("Trousseau data store imported: %s", importedFilePath))
}

func ListRecipientsAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments provided to 'list-recipients' command")
	}

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	recipients := store.Meta.ListRecipients()
	for _, r := range recipients {
		trousseau.Logger.Info(r)
	}
}

func AddRecipientAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'add-recipient' command")
	}

	recipient := c.Args()[0]

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.Meta.AddRecipient(recipient)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Encrypt(store)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Write(trousseau.InferStorePath())
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

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.Meta.RemoveRecipient(recipient)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Encrypt(store)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Write(trousseau.InferStorePath())
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

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	value, err := store.Data.Get(c.Args()[0])
	if err != nil {
		log.Fatal(err)
	}

	// If the --file flag is provided
	if c.String("file") != "" {
		valueBytes, ok := value.(string)
		if !ok {
			log.Fatal(fmt.Sprintf("unable to write %s value to file", c.Args()[0]))
		}

		err := ioutil.WriteFile(c.String("file"), []byte(valueBytes), os.FileMode(0644))
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

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	store.Data.Set(key, value)

	err = tr.Encrypt(store)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Write(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		trousseau.Logger.Info(fmt.Sprintf("%s:%s", key, value))
	}
}

func RenameAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 2) {
		log.Fatal("Incorrect number of arguments provided to 'rename' command")
	}

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	err = store.Data.Rename(c.Args()[0], c.Args()[1], c.Bool("overwrite"))
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Encrypt(store)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Write(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	if c.Bool("verbose") == true {
		trousseau.Logger.Info(fmt.Sprintf("renamed: %s to %s", c.Args()[0], c.Args()[1]))
	}
}

func DelAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 1) {
		log.Fatal("Incorrect number of arguments to 'del' command")
	}

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	store.Data.Del(c.Args()[0])

	tr.Encrypt(store)
	if err != nil {
		log.Fatal(err)
	}

	err = tr.Write(trousseau.InferStorePath())
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

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	keys := store.Data.Keys()
	for _, k := range keys {
		trousseau.Logger.Info(k)
	}
}

func ShowAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'show' command")
	}

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	items := store.Data.Items()
	for k, v := range items {
		trousseau.Logger.Info(fmt.Sprintf("%s : %s", k, v.(string)))
	}
}

func MetaAction(c *libcli.Context) {
	if !hasExpectedArgs(c.Args(), 0) {
		log.Fatal("Incorrect number of arguments to 'meta' command")
	}

	tr, err := trousseau.OpenTrousseau(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		log.Fatal(err)
	}

	trousseau.Logger.Info(store.Meta)
}

func UpgradeAction(c *libcli.Context) {
	data, err := ioutil.ReadFile(trousseau.InferStorePath())
	if err != nil {
		log.Fatal(err)
	}

	newStoreFile, err := trousseau.UpgradeFrom("0.3.0", data, trousseau.UpgradeClosures)
	if err != nil {
		log.Fatal(err)
	}

	err = ioutil.WriteFile(trousseau.InferStorePath(), newStoreFile, os.FileMode(0700))
	if err != nil {
		log.Fatal(err)
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
