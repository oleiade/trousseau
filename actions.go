package trousseau

import (
	"time"

	"github.com/oleiade/trousseau/dsn"
	"os"
	"io"
	"io/ioutil"
	"encoding/json"
	"fmt"
	"strings"
)

func CreateAction(recipients []string) {
	meta := Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: TROUSSEAU_VERSION,
	}
	store := NewStore(meta)

	tr := Trousseau{
		CryptoType:      ASYMMETRIC_ENCRYPTION,
		CryptoAlgorithm: GPG_ENCRYPTION,
	}

	err := tr.Encrypt(store)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	InfoLogger.Println("Trousseau data store succesfully created")
}

func PushAction(destination string, sshPrivateKey string, askPassword bool) {
	endpointDsn, err := dsn.Parse(destination)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(S3Defaults)
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		err = UploadUsingS3(endpointDsn)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
		InfoLogger.Println("Trousseau data store succesfully pushed to s3")
	case "scp":
		err := endpointDsn.SetDefaults(ScpDefaults)
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		if askPassword == true {
			password := PromptForPassword()
			endpointDsn.Secret = password
		}

		err = UploadUsingScp(endpointDsn, sshPrivateKey)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
		InfoLogger.Println("Trousseau data store succesfully pushed to ssh remote storage")
	case "gist":
		err = UploadUsingGist(endpointDsn)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
		InfoLogger.Println("Trousseau data store succesfully pushed to gist")
	}
}

func PullAction(source string, sshPrivateKey string, askPassword bool) {
	endpointDsn, err := dsn.Parse(source)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(S3Defaults)
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		err = DownloadUsingS3(endpointDsn)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
		InfoLogger.Println("Trousseau data store succesfully pulled from S3")
	case "scp":
		err := endpointDsn.SetDefaults(ScpDefaults)
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		if askPassword == true {
			password := PromptForPassword()
			endpointDsn.Secret = password
		}

		err = DownloadUsingScp(endpointDsn, sshPrivateKey)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
		InfoLogger.Println("Trousseau data store succesfully pulled from ssh remote storage")
	case "gist":
		err = DownloadUsingGist(endpointDsn)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
		InfoLogger.Println("Trousseau data store succesfully pulled from gist")
	default:
		if endpointDsn.Scheme == "" {
			ErrorLogger.Fatalf("No dsn scheme supplied")
		} else {
			ErrorLogger.Fatalf("Invalid dsn scheme supplied: %s", endpointDsn.Scheme)
		}
	}

	InfoLogger.Println("Trousseau data store succesfully pulled from remote storage")
}

func ExportAction(to string, plain bool) {
	outputFile, err := os.Create(to)
	if err != nil {
		ErrorLogger.Fatal(err)
	}
	defer outputFile.Close()

	// Make sure the file is readble/writable only
	// by its owner
	err = os.Chmod(outputFile.Name(), os.FileMode(0600))
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	if plain == true {
		tr, err := OpenTrousseau(InferStorePath())
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		store, err := tr.Decrypt()
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		storeBytes, err := json.Marshal(store)
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		err = ioutil.WriteFile(to, storeBytes, os.FileMode(0600))
		if err != nil {
			ErrorLogger.Fatal(err)
		}
	} else {
		inputFile, err := os.Open(InferStorePath())
		defer inputFile.Close()
		if err != nil {
			ErrorLogger.Fatal(err)
		}

		_, err = io.Copy(outputFile, inputFile)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
	}

	InfoLogger.Println(fmt.Sprintf("Trousseau data store exported to: %s", to))
}

func ImportAction(from string, strategy ImportStrategy, plain bool) {
	var importedStore *Store = &Store{}
	var localFilePath string = InferStorePath()

	localTr, err := OpenTrousseau(localFilePath)
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	localStore, err := localTr.Decrypt()
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	if plain == true {
	importedData, err := ioutil.ReadFile(from)
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	err = json.Unmarshal(importedData, importedStore)
	if err != nil {
	ErrorLogger.Fatal(err)
	}
	} else {
	importedTr, err := OpenTrousseau(from)
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	importedStore, err = importedTr.Decrypt()
	if err != nil {
	ErrorLogger.Fatal(err)
	}
	}

	err = ImportStore(importedStore, localStore, strategy)
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	err = localTr.Encrypt(localStore)
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	err = localTr.Write(localFilePath)
	if err != nil {
	ErrorLogger.Fatal(err)
	}

	InfoLogger.Println(fmt.Sprintf("Trousseau data store imported: %s", from))
}

func ListRecipientsAction() {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	recipients := store.Meta.ListRecipients()
	for _, r := range recipients {
		InfoLogger.Println(r)
	}
}

func AddRecipientAction(recipient string) {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = store.Meta.AddRecipient(recipient)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Encrypt(store)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}
}

func RemoveRecipientAction(recipient string) {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = store.Meta.RemoveRecipient(recipient)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Encrypt(store)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}
}

func GetAction(key string, filepath string) {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	value, err := store.Data.Get(key)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	// If the --file flag is provided
	if filepath != "" {
		valueBytes, ok := value.(string)
		if !ok {
			ErrorLogger.Fatal(fmt.Sprintf("unable to write %s value to file", key))
		}

		err := ioutil.WriteFile(filepath, []byte(valueBytes), os.FileMode(0644))
		if err != nil {
			ErrorLogger.Fatal(err)
		}
	} else {
		// Use fmt.Print to support patch processes,
		// to get the value "as is" without any appended newlines
		if isPipe(os.Stdout) {
			fmt.Print(value)
		} else {
			InfoLogger.Println(value)
		}
	}
}

func SetAction(key, value, file string) {
	// If the --file flag is provided
	if file != "" {
		// And the file actually exists on file system
		if PathExists(file) {
			// Then load it's content
			fileContent, err := ioutil.ReadFile(file)
			if err != nil {
				ErrorLogger.Fatal(err)
			}

			value = string(fileContent)
		} else {
			ErrorLogger.Fatalf("Cannot open %s because it doesn't exist", file)
		}
	}

	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store.Data.Set(key, value)

	err = tr.Encrypt(store)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}
}

func RenameAction(src, dest string, overwrite bool) {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = store.Data.Rename(src, dest, overwrite)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Encrypt(store)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

}

func DelAction(key string) {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store.Data.Del(key)

	tr.Encrypt(store)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}
}

func KeysAction() {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	keys := store.Data.Keys()
	for _, k := range keys {
		InfoLogger.Println(k)
	}
}

func ShowAction() {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	items := store.Data.Items()
	for k, v := range items {
		InfoLogger.Println(fmt.Sprintf("%s : %s", k, v.(string)))
	}
}

func MetaAction() {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	store, err := tr.Decrypt()
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	InfoLogger.Println(store.Meta)
}

func UpgradeAction(yes, noBackup bool) {
	var proceed string = "n"

	data, err := ioutil.ReadFile(InferStorePath())
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	version := DiscoverVersion(data, VersionDiscoverClosures)
	if version == "" {
		fmt.Errorf("Initial store version could not be detected")
	}

	newStoreFile, err := UpgradeFrom(version, data, UpgradeClosures)
	if err != nil {
		ErrorLogger.Fatal(err)
	}

	if yes == false {
		fmt.Printf("You are about to upgrade trousseau data "+
					"store %s (version %s) up to version %s. Proceed? [Y/n] ",
			InferStorePath(), version, TROUSSEAU_VERSION)
		_, err = fmt.Scanf("%s", &proceed)
		if err != nil {
			ErrorLogger.Fatal(err)
		}
	}

	if strings.ToLower(proceed) == "y" || yes {
		// Write a backup of the old store file inplace
		if noBackup == false {
			err = ioutil.WriteFile(InferStorePath()+".bkp", data, os.FileMode(0700))
			if err != nil {
				ErrorLogger.Fatal(err)
			}
		}

		// Overwrite source legacy store with the new version content
		err = ioutil.WriteFile(InferStorePath(), newStoreFile, os.FileMode(0700))
		if err != nil {
			ErrorLogger.Fatal(err)
		}
	} else {
		fmt.Println("upgrade cancelled")
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

