package trousseau

import (
	"time"

	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"strings"

	"github.com/oleiade/trousseau/dsn"
)

func CreateAction(ct CryptoType, ca CryptoAlgorithm, recipients []string) error {
	meta := Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: TROUSSEAU_VERSION,
	}
	store := NewStore(meta)

	tr := Trousseau{
		CryptoType:      ct,
		CryptoAlgorithm: ca,
	}

	if tr.CryptoType == SYMMETRIC_ENCRYPTION {
		passphrase, err := GetPassphrase()
		if err != nil {

			if !AskPassphraseFlagCheck() {
				AskPassphrase(true)
			}
		} else {
			SetPassphrase(passphrase)
		}
	}

	err := tr.Encrypt(store)
	if err != nil {
		return err
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		return err
	}

	return nil
}

func PushAction(destination string, sshPrivateKey string, askPassword bool) error {
	endpointDsn, err := dsn.Parse(destination)
	if err != nil {
		return err
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(S3Defaults)
		if err != nil {
			return err
		}

		err = UploadUsingS3(endpointDsn)
		if err != nil {
			return err
		}
	case "scp":
		err := endpointDsn.SetDefaults(ScpDefaults)
		if err != nil {
			return err
		}

		if askPassword == true {
			password := PromptForHiddenInput("Ssh endpoint password: ")
			endpointDsn.Secret = password
		}

		err = UploadUsingScp(endpointDsn, sshPrivateKey)
		if err != nil {
			return err
		}
	case "gist":
		err = UploadUsingGist(endpointDsn)
		if err != nil {
			return err
		}
	}

	return nil
}

func PullAction(source string, sshPrivateKey string, askPassword bool) error {
	endpointDsn, err := dsn.Parse(source)
	if err != nil {
		return err
	}

	switch endpointDsn.Scheme {
	case "s3":
		err := endpointDsn.SetDefaults(S3Defaults)
		if err != nil {
			return err
		}

		err = DownloadUsingS3(endpointDsn)
		if err != nil {
			return err
		}
	case "scp":
		err := endpointDsn.SetDefaults(ScpDefaults)
		if err != nil {
			return err
		}

		if askPassword == true {
			password := PromptForHiddenInput("Ssh endpoint password: ")
			endpointDsn.Secret = password
		}

		err = DownloadUsingScp(endpointDsn, sshPrivateKey)
		if err != nil {
			return err
		}
	case "gist":
		err = DownloadUsingGist(endpointDsn)
		if err != nil {
			return err
		}
	default:
		if endpointDsn.Scheme == "" {
			ErrorLogger.Fatalf("No dsn scheme supplied")
		} else {
			ErrorLogger.Fatalf("Invalid dsn scheme supplied: %s", endpointDsn.Scheme)
		}
	}

	return nil
}

func ExportAction(destination io.Writer, plain bool) error {
	if plain == true {
		tr, err := OpenTrousseau(InferStorePath())
		if err != nil {
			return err
		}

		store, err := tr.Decrypt()
		if err != nil {
			return err
		}

		storeBytes, err := json.Marshal(store)
		if err != nil {
			return err
		}

		_, err = destination.Write(storeBytes)
		if err != nil {
			return err
		}
	} else {
		inputFile, err := os.Open(InferStorePath())
		defer inputFile.Close()
		if err != nil {
			return err
		}

		_, err = io.Copy(destination, inputFile)
		if err != nil {
			return err
		}
	}

	return nil
}

func ImportAction(source io.Reader, strategy ImportStrategy, plain bool) error {
	var data []byte
	var err error
	var importedStore *Store = &Store{}
	var localFilePath string = InferStorePath()

	localTr, err := OpenTrousseau(localFilePath)
	if err != nil {
		return err
	}

	localStore, err := localTr.Decrypt()
	if err != nil {
		return err
	}

	if plain == true {
		data, err = ioutil.ReadAll(source)
		if err != nil {
			return err
		}

		err = json.Unmarshal(data, importedStore)
		if err != nil {
			return err
		}
	} else {
		data, err = ioutil.ReadAll(source)
		if err != nil {
			return err
		}

		importedTr, err := FromBytes(data)
		if err != nil {
			return err
		}

		importedStore, err = importedTr.Decrypt()
		if err != nil {
			return err
		}
	}

	err = ImportStore(importedStore, localStore, strategy)
	if err != nil {
		return err
	}

	err = localTr.Encrypt(localStore)
	if err != nil {
		return err
	}

	err = localTr.Write(localFilePath)
	if err != nil {
		return err
	}

	return nil
}

func ListRecipientsAction() error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	recipients := store.Meta.ListRecipients()
	for _, r := range recipients {
		InfoLogger.Println(r)
	}

	return nil
}

func AddRecipientAction(recipient string) error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	err = store.Meta.AddRecipient(recipient)
	if err != nil {
		return err
	}

	err = tr.Encrypt(store)
	if err != nil {
		return err
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		return err
	}

	return nil
}

func RemoveRecipientAction(recipient string) error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	err = store.Meta.RemoveRecipient(recipient)
	if err != nil {
		return err
	}

	err = tr.Encrypt(store)
	if err != nil {
		return err
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		return err
	}

	return nil
}

func GetAction(key string, filepath string) error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	value, err := store.Data.Get(key)
	if err != nil {
		return err
	}

	// If the --file flag is provided
	if filepath != "" {
		valueBytes, ok := value.(string)
		if !ok {
			ErrorLogger.Fatal(fmt.Sprintf("unable to write %s value to file", key))
		}

		err := ioutil.WriteFile(filepath, []byte(valueBytes), os.FileMode(0600))
		if err != nil {
			return err
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

	return nil
}

func SetAction(key, value, file string) error {
	// If the --file flag is provided
	if file != "" {
		// And the file actually exists on file system
		if PathExists(file) {
			// Then load it's content
			fileContent, err := ioutil.ReadFile(file)
			if err != nil {
				return err
			}

			value = string(fileContent)
		} else {
			ErrorLogger.Fatalf("Cannot open %s because it doesn't exist", file)
		}
	}

	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	store.Data.Set(key, value)

	err = tr.Encrypt(store)
	if err != nil {
		return err
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		return err
	}

	return nil
}

func RenameAction(src, dest string, overwrite bool) error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	err = store.Data.Rename(src, dest, overwrite)
	if err != nil {
		return err
	}

	err = tr.Encrypt(store)
	if err != nil {
		return err
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		return err
	}

	return nil
}

func DelAction(key string) error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	store.Data.Del(key)

	tr.Encrypt(store)
	if err != nil {
		return err
	}

	err = tr.Write(InferStorePath())
	if err != nil {
		return err
	}

	return nil
}

func KeysAction() error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	keys := store.Data.Keys()
	for _, k := range keys {
		InfoLogger.Println(k)
	}

	return nil
}

func ShowAction() error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	items := store.Data.Items()
	for k, v := range items {
		InfoLogger.Println(fmt.Sprintf("%s : %s", k, v.(string)))
	}

	return nil
}

func MetaAction() error {
	tr, err := OpenTrousseau(InferStorePath())
	if err != nil {
		return err
	}

	store, err := tr.Decrypt()
	if err != nil {
		return err
	}

	InfoLogger.Println(store.Meta)
	return nil
}

func UpgradeAction(yes, noBackup bool) error {
	var proceed string = "n"

	data, err := ioutil.ReadFile(InferStorePath())
	if err != nil {
		return err
	}

	version := DiscoverVersion(data, VersionDiscoverClosures)
	if version == "" {
		fmt.Errorf("Initial store version could not be detected")
	}

	newStoreFile, err := UpgradeFrom(version, data, UpgradeClosures)
	if err != nil {
		return err
	}

	if yes == false {
		fmt.Printf("You are about to upgrade trousseau data "+
			"store %s (version %s) up to version %s. Proceed? [Y/n] ",
			InferStorePath(), version, TROUSSEAU_VERSION)
		count, _ := fmt.Scanf("%s", &proceed)

		// Default to "y" if return was pressed
		if count == 0 {
			proceed = "y"
		}
	}

	if strings.ToLower(proceed) == "y" || yes {
		// Write a backup of the old store file inplace
		if noBackup == false {
			err = ioutil.WriteFile(InferStorePath()+".bkp", data, os.FileMode(0700))
			if err != nil {
				return err
			}
		}

		// Overwrite source legacy store with the new version content
		err = ioutil.WriteFile(InferStorePath(), newStoreFile, os.FileMode(0700))
		if err != nil {
			return err
		}
	} else {
		fmt.Println("upgrade cancelled")
	}

	return nil
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
