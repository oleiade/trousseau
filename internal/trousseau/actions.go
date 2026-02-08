package trousseau

import (
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"strings"

	"github.com/oleiade/trousseau/internal/config"

	"github.com/oleiade/trousseau/pkg/dsn"
)

func CreateAction(ct CryptoType, ca CryptoAlgorithm, recipients []string) error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	secretStore := NewSecretStore()
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

	err = vault.Lock(config, secretStore)
	if err != nil {
		return err
	}

	f, err := os.OpenFile(InferStorePath(config), os.O_CREATE|os.O_WRONLY, 0700)
	if err != nil {
		return err
	}

	err = vault.Dump(f)
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
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	if plain == true {
		vault, err := OpenVault(InferStorePath(config))
		if err != nil {
			return err
		}

		secretStore, err := vault.Unlock(config)
		if err != nil {
			return err
		}

		storeBytes, err := json.Marshal(secretStore)
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
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	localFilePath := InferStorePath(config)
	localVault, err := OpenVault(localFilePath)
	if err != nil {
		return err
	}

	localSecretStore, err := localVault.Unlock(config)
	if err != nil {
		return err
	}

	importedStore := NewSecretStore()
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
		importedVault, err := ReadVault(source)
		if err != nil {
			return err
		}

		importedStore, err = importedVault.Unlock(config)
		if err != nil {
			return err
		}
	}

	err = ImportStore(importedStore, localSecretStore, strategy)
	if err != nil {
		return err
	}

	err = localVault.Lock(config, localSecretStore)
	if err != nil {
		return err
	}

	f, err := os.Open(localFilePath)
	if err != nil {
		return err
	}

	err = localVault.Dump(f)
	if err != nil {
		return err
	}

	return nil
}

func ListRecipientsAction() error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	for _, r := range secretStore.Metadata.Recipients {
		InfoLogger.Println(r)
	}

	return nil
}

func AddRecipientAction(recipient string) error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	for _, r := range secretStore.Metadata.Recipients {
		if r == recipient {
			return fmt.Errorf("recipient %s already present", recipient)
		}
	}
	secretStore.Metadata.Recipients = append(secretStore.Metadata.Recipients, recipient)

	err = vault.Lock(config, secretStore)
	if err != nil {
		return err
	}

	f, err := os.Open(InferStorePath(config))
	if err != nil {
		return err
	}

	err = vault.Dump(f)
	if err != nil {
		return err
	}

	return nil
}

func RemoveRecipientAction(recipient string) error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	var filteredRecipients []string
	for _, r := range secretStore.Metadata.Recipients {
		if r != recipient {
			filteredRecipients = append(filteredRecipients, recipient)
		}
	}
	secretStore.Metadata.Recipients = filteredRecipients

	err = vault.Lock(config, secretStore)
	if err != nil {
		return err
	}

	f, err := os.Open(InferStorePath(config))
	if err != nil {
		return err
	}

	err = vault.Dump(f)
	if err != nil {
		return err
	}

	return nil
}

func GetAction(key string, filepath string) error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	value, found := secretStore.Data[key]
	if !found {
		return fmt.Errorf("key %s not found", key)
	}

	// If the --file flag is provided
	if filepath != "" {
		err := ioutil.WriteFile(filepath, []byte(value), os.FileMode(0600))
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
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
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

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}
	secretStore.Data[key] = value

	err = vault.Lock(config, secretStore)
	if err != nil {
		return err
	}

	f, err := os.Open(InferStorePath(config))
	if err != nil {
		return err
	}

	err = vault.Dump(f)
	if err != nil {
		return err
	}

	return nil
}

func RenameAction(src, dest string, overwrite bool) error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	srcValue, found := secretStore.Data[src]
	if !found {
		return fmt.Errorf("key %s not found", src)
	}

	// If destination key already exists, and overwrite flag is
	// set to false, then return an error
	_, found = secretStore.Data[dest]
	if found && overwrite == false {
		return fmt.Errorf("key %s already exists. The overwrite flag was not set", dest)
	}

	secretStore.Data[dest] = srcValue
	delete(secretStore.Data, src)

	err = vault.Lock(config, secretStore)
	if err != nil {
		return err
	}

	f, err := os.Open(InferStorePath(config))
	if err != nil {
		return err
	}

	err = vault.Dump(f)
	if err != nil {
		return err
	}

	return nil
}

func DelAction(key string) error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	delete(secretStore.Data, key)

	vault.Lock(config, secretStore)
	if err != nil {
		return err
	}

	f, err := os.Open(InferStorePath(config))
	if err != nil {
		return err
	}

	err = vault.Dump(f)
	if err != nil {
		return err
	}

	return nil
}

func KeysAction() error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	for k := range secretStore.Data {
		InfoLogger.Println(k)
	}

	return nil
}

func ShowAction() error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	for k, v := range secretStore.Data {
		InfoLogger.Println(fmt.Sprintf("%s : %s", k, v))
	}

	return nil
}

func MetaAction() error {
	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	vault, err := OpenVault(InferStorePath(config))
	if err != nil {
		return err
	}

	secretStore, err := vault.Unlock(config)
	if err != nil {
		return err
	}

	InfoLogger.Printf("%+v\n", secretStore.Metadata)
	return nil
}

func UpgradeAction(yes, noBackup bool) error {
	var proceed string = "n"

	config, err := config.Load("")
	if err != nil {
		return fmt.Errorf("unable to load configuration; reason: %s", err.Error())
	}

	data, err := ioutil.ReadFile(InferStorePath(config))
	if err != nil {
		return err
	}

	version := DiscoverVersion(data, VersionDiscoverClosures)
	if version == "" {
		return fmt.Errorf("Initial store version could not be detected")
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
