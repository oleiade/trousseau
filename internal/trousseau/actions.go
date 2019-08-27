package trousseau

import (
	"time"

	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"strings"

	"github.com/oleiade/trousseau/internal/config"
	"github.com/oleiade/trousseau/internal/store"

	"github.com/oleiade/trousseau/pkg/dsn"
)

func CreateAction(ct CryptoType, ca CryptoAlgorithm, recipients []string) error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	meta := store.Meta{
		CreatedAt:        time.Now().String(),
		LastModifiedAt:   time.Now().String(),
		Recipients:       recipients,
		TrousseauVersion: TROUSSEAU_VERSION,
	}
	store := store.NewStore(meta)

	vault := Vault{
		CryptoType:      ct,
		CryptoAlgorithm: ca,
	}

	if vault.CryptoType == SYMMETRIC_ENCRYPTION {
		passphrase, err := GetPassphrase(config)
		if err != nil {

			if !AskPassphraseFlagCheck() {
				AskPassphrase(true)
			}
		} else {
			SetPassphrase(passphrase)
		}
	}

	err = vault.Encrypt(config, store)
	if err != nil {
		return err
	}

	err = vault.Write(InferStorePath(config))
	if err != nil {
		return err
	}

	return nil
}

func PushAction(destination string, sshPrivateKey string, askPassword bool) error {
	endpointDSN, err := dsn.Parse(destination)
	if err != nil {
		return err
	}

	var remote UploadDownloader

	switch endpointDSN.Scheme {
	case "s3":
		err := endpointDSN.SetDefaults(S3Defaults)
		if err != nil {
			return err
		}

		remote, err = NewS3Remote(endpointDSN.Port, endpointDSN.Id, endpointDSN.Secret, endpointDSN.Host)
		if err != nil {
			return err
		}
	case "scp":
		err := endpointDSN.SetDefaults(ScpDefaults)
		if err != nil {
			return err
		}

		if askPassword == true {
			password := PromptForHiddenInput("Ssh endpoint password: ")
			endpointDSN.Secret = password
		}

		remote = NewSCPRemote(endpointDSN.Host, endpointDSN.Port, endpointDSN.Id, endpointDSN.Secret, sshPrivateKey)
	case "gist":
		remote = NewGistRemote(endpointDSN.Id, endpointDSN.Secret)
	}

	err = remote.Upload(endpointDSN.Path)
	if err != nil {
		return err
	}

	return nil
}

func PullAction(source string, sshPrivateKey string, askPassword bool) error {
	endpointDSN, err := dsn.Parse(source)
	if err != nil {
		return err
	}

	var remote UploadDownloader

	switch endpointDSN.Scheme {
	case "s3":
		err := endpointDSN.SetDefaults(S3Defaults)
		if err != nil {
			return err
		}

		remote, err = NewS3Remote(endpointDSN.Port, endpointDSN.Id, endpointDSN.Secret, endpointDSN.Host)
		if err != nil {
			return err
		}
	case "scp":
		err := endpointDSN.SetDefaults(ScpDefaults)
		if err != nil {
			return err
		}

		if askPassword == true {
			password := PromptForHiddenInput("Ssh endpoint password: ")
			endpointDSN.Secret = password
		}

		remote = NewSCPRemote(endpointDSN.Host, endpointDSN.Port, endpointDSN.Id, endpointDSN.Secret, sshPrivateKey)
	case "gist":
		remote = NewGistRemote(endpointDSN.Id, endpointDSN.Secret)
	default:
		if endpointDSN.Scheme == "" {
			ErrorLogger.Fatalf("No dsn scheme supplied")
		} else {
			ErrorLogger.Fatalf("Invalid dsn scheme supplied: %s", endpointDSN.Scheme)
		}
	}

	err = remote.Download(endpointDSN.Path)
	if err != nil {
		return err
	}

	return nil
}

func ExportAction(destination io.Writer, plain bool) error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	if plain == true {
		vault, err := OpenTrousseau(InferStorePath(config))
		if err != nil {
			return err
		}

		store, err := vault.Decrypt(config)
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
		inputFile, err := os.Open(InferStorePath(config))
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

	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	var importedStore *store.Store = &store.Store{}
	var localFilePath string = InferStorePath(config)

	localTr, err := OpenTrousseau(localFilePath)
	if err != nil {
		return err
	}

	localStore, err := localTr.Decrypt(config)
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

		importedStore, err = importedTr.Decrypt(config)
		if err != nil {
			return err
		}
	}

	err = ImportStore(importedStore, localStore, strategy)
	if err != nil {
		return err
	}

	err = localTr.Encrypt(config, localStore)
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
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
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
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
	if err != nil {
		return err
	}

	err = store.Meta.AddRecipient(recipient)
	if err != nil {
		return err
	}

	err = vault.Encrypt(config, store)
	if err != nil {
		return err
	}

	err = vault.Write(InferStorePath(config))
	if err != nil {
		return err
	}

	return nil
}

func RemoveRecipientAction(recipient string) error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
	if err != nil {
		return err
	}

	err = store.Meta.RemoveRecipient(recipient)
	if err != nil {
		return err
	}

	err = vault.Encrypt(config, store)
	if err != nil {
		return err
	}

	err = vault.Write(InferStorePath(config))
	if err != nil {
		return err
	}

	return nil
}

func GetAction(key string, filepath string) error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
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
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

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

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
	if err != nil {
		return err
	}

	store.Data.Set(key, value)

	err = vault.Encrypt(config, store)
	if err != nil {
		return err
	}

	err = vault.Write(InferStorePath(config))
	if err != nil {
		return err
	}

	return nil
}

func RenameAction(src, dest string, overwrite bool) error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
	if err != nil {
		return err
	}

	err = store.Data.Rename(src, dest, overwrite)
	if err != nil {
		return err
	}

	err = vault.Encrypt(config, store)
	if err != nil {
		return err
	}

	err = vault.Write(InferStorePath(config))
	if err != nil {
		return err
	}

	return nil
}

func DelAction(key string) error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
	if err != nil {
		return err
	}

	store.Data.Del(key)

	vault.Encrypt(config, store)
	if err != nil {
		return err
	}

	err = vault.Write(InferStorePath(config))
	if err != nil {
		return err
	}

	return nil
}

func KeysAction() error {
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
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
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
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
	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenTrousseau(InferStorePath(config))
	if err != nil {
		return err
	}

	store, err := vault.Decrypt(config)
	if err != nil {
		return err
	}

	InfoLogger.Println(store.Meta)
	return nil
}

func UpgradeAction(yes, noBackup bool) error {
	var proceed string = "n"

	config, err := config.Load("")
	if err != nil {
		fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	data, err := ioutil.ReadFile(InferStorePath(config))
	if err != nil {
		return err
	}

	version := DiscoverVersion(data, VersionDiscoverClosures)
	if version == "" {
		fmt.Errorf("Initial store version could not be detected")
	}

	newStoreFile, err := UpgradeFrom(config, version, data, UpgradeClosures)
	if err != nil {
		return err
	}

	if yes == false {
		fmt.Printf("You are about to upgrade trousseau data "+
			"store %s (version %s) up to version %s. Proceed? [Y/n] ",
			InferStorePath(config), version, TROUSSEAU_VERSION)
		count, _ := fmt.Scanf("%s", &proceed)

		// Default to "y" if return was pressed
		if count == 0 {
			proceed = "y"
		}
	}

	if strings.ToLower(proceed) == "y" || yes {
		// Write a backup of the old store file inplace
		if noBackup == false {
			err = ioutil.WriteFile(InferStorePath(config)+".bkp", data, os.FileMode(0700))
			if err != nil {
				return err
			}
		}

		// Overwrite source legacy store with the new version content
		err = ioutil.WriteFile(InferStorePath(config), newStoreFile, os.FileMode(0700))
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
