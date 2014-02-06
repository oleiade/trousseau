package trousseau

import (
    "github.com/codegangsta/cli"
)

func CreateCommand() cli.Command {
    return cli.Command{
        Name:   "create",
        Usage:  "create trousseau",
        Action: CreateAction,
    }
}

func PushCommand() cli.Command {
    return cli.Command{
        Name:   "push",
        Usage:  "pushes the trousseau to remote storage",
        Action: PushAction,
        Flags: []cli.Flag{
            OverwriteFlag(),
            SshPrivateKeyPathFlag(),
        },
    }
}

func PullCommand() cli.Command {
    return cli.Command{
        Name:   "pull",
        Usage:  "pull the trousseau from remote storage",
        Action: PullAction,
        Flags: []cli.Flag{
            OverwriteFlag(),
            SshPrivateKeyPathFlag(),
        },
    }
}

func ExportCommand() cli.Command {
    return cli.Command{
        Name:   "export",
        Usage:  "export the encrypted trousseau to local fs",
        Action: ExportAction,
        Flags: []cli.Flag{
            OverwriteFlag(),
        },
    }
}

func ImportCommand() cli.Command {
    return cli.Command{
        Name:   "import",
        Usage:  "import an encrypted trousseau from local fs",
        Action: ImportAction,
        Flags: []cli.Flag{
            OverwriteFlag(),
            TheirsFlag(),
            YoursFlag(),
        },
    }
}

func AddRecipientCommand() cli.Command {
    return cli.Command{
        Name:   "add-recipient",
        Usage:  "add a recipient to the encrypted trousseau",
        Action: AddRecipientAction,
    }
}

func RemoveRecipientCommand() cli.Command {
    return cli.Command{
        Name:   "remove-recipient",
        Usage:  "remove a recipient of the encrypted trousseau",
        Action: RemoveRecipientAction,
    }
}

func SetCommand() cli.Command {
    return cli.Command{
        Name:   "set",
        Usage:  "sets a key value pair in the store",
        Action: SetAction,
    }
}

func GetCommand() cli.Command {
    return cli.Command{
        Name:   "get",
        Usage:  "get a value from the trousseau",
        Action: GetAction,
    }
}

func DelCommand() cli.Command {
    return cli.Command{
        Name:   "del",
        Usage:  "delete the point key pair from the store",
        Action: DelAction,
    }
}

func KeysCommand() cli.Command {
    return cli.Command{
        Name:   "keys",
        Usage:  "Lists the store keys",
        Action: KeysAction,
    }
}

func ShowCommand() cli.Command {
    return cli.Command{
        Name:   "show",
        Usage:  "shows trousseau content",
        Action: ShowAction,
    }
}

func MetaCommand() cli.Command {
    return cli.Command{
        Name:   "meta",
        Usage:  "shows trousseau metadata",
        Action: MetaAction,
    }
}
