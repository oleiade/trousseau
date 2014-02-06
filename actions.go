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

    fmt.Println("Trousseau data store succesfully created")
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
        fmt.Printf("Trousseau data store succesfully pushed to s3")
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
        fmt.Printf("Trousseau data store succesfully pushed to ssh remote storage")
    case "gist":
        err = uploadUsingGist(endpointDsn)
        if err != nil {
            log.Fatal(err)
        }
        fmt.Printf("Trousseau data store succesfully pushed to gist")
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
        fmt.Printf("Trousseau data store succesfully pulled from S3")
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
        fmt.Printf("Trousseau data store succesfully pulled from ssh remote storage")
    case "gist":
        err = DownloadUsingGist(endpointDsn)
        if err != nil {
            log.Fatal(err)
        }
        fmt.Printf("Trousseau data store succesfully pulled from gist")
    default:
        if endpointDsn.Scheme == "" {
            log.Fatalf("No dsn scheme supplied")
        } else {
            log.Fatalf("Invalid dsn scheme supplied: %s", endpointDsn.Scheme)
        }
    }

    fmt.Printf("Trousseau data store succesfully pulled from remote storage")
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

    fmt.Printf("Trousseau data store exported to %s", outputFilePath)
}

func ImportAction(c *cli.Context) {
    if !hasExpectedArgs(c.Args(), 1) {
        log.Fatal("Incorrect number of arguments to 'import' command")
    }

    var err error
    var inputFilePath string = c.Args()[0]
    var outputFilePath string = gStorePath
    var strategy *ImportStrategy = new(ImportStrategy)

    // Transform provided merging startegy flags
    // into a proper ImportStrategy byte.
    err = strategy.FromCliContext(c)
    if err != nil {
        log.Fatal(err)
    }

    importedStore, err := NewEncryptedStoreFromFile(inputFilePath, c.GlobalString("passphrase"))
    if err != nil {
        log.Fatal(err)
    }

    localStore, err := NewEncryptedStoreFromFile(outputFilePath, c.GlobalString("passphrase"))
    if err != nil {
        log.Fatal(err)
    }

    err = importedStore.Decrypt()
    if err != nil {
        log.Fatal(err)
    }

    err = localStore.Decrypt()
    if err != nil {
        log.Fatal(err)
    }

    err = ImportStore(importedStore, localStore, *strategy)
    if err != nil {
        log.Fatal(err)
    }

    err = localStore.Encrypt()
    if err != nil {
        log.Fatal(err)
    }

    err = localStore.WriteToFile(outputFilePath)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Trousseau data store imported")
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

    fmt.Printf("%s recipient added to trousseau data store", recipient)
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

    fmt.Printf("%s recipient removed from trousseau data store", recipient)
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
