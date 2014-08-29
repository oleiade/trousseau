package cli

import (
	libcli "github.com/codegangsta/cli"
)

func CreateCommand() libcli.Command {
	return libcli.Command{
		Name:   "create",
		Usage:  "create the trousseau data store",
		Action: CreateAction,
	}
}

func PushCommand() libcli.Command {
	return libcli.Command{
		Name:   "push",
		Usage:  "pushes the trousseau to remote storage",
		Action: PushAction,
		Flags: []libcli.Flag{
			OverwriteFlag(),
			AskPassword(),
			VerboseFlag(),
			SshPrivateKeyPathFlag(),
		},
	}
}

func PullCommand() libcli.Command {
	return libcli.Command{
		Name:   "pull",
		Usage:  "pull the trousseau from remote storage",
		Action: PullAction,
		Flags: []libcli.Flag{
			OverwriteFlag(),
			AskPassword(),
			VerboseFlag(),
			SshPrivateKeyPathFlag(),
		},
	}
}

func ExportCommand() libcli.Command {
	return libcli.Command{
		Name:   "export",
		Usage:  "export the encrypted trousseau to local fs",
		Action: ExportAction,
		Flags: []libcli.Flag{
			OverwriteFlag(),
			PlainFlag(),
			VerboseFlag(),
		},
	}
}

func ImportCommand() libcli.Command {
	return libcli.Command{
		Name:   "import",
		Usage:  "import an encrypted trousseau from local fs",
		Action: ImportAction,
		Flags: []libcli.Flag{
			VerboseFlag(),
			PlainFlag(),
			OverwriteFlag(),
			TheirsFlag(),
			YoursFlag(),
		},
	}
}

func ListRecipientsCommand() libcli.Command {
	return libcli.Command{
		Name:   "list-recipients",
		Usage:  "lists trousseau data store recipients",
		Action: ListRecipientsAction,
		Flags: []libcli.Flag{
			VerboseFlag(),
		},
	}
}

func AddRecipientCommand() libcli.Command {
	return libcli.Command{
		Name:   "add-recipient",
		Usage:  "add a recipient to the encrypted trousseau",
		Action: AddRecipientAction,
		Flags: []libcli.Flag{
			VerboseFlag(),
		},
	}
}

func RemoveRecipientCommand() libcli.Command {
	return libcli.Command{
		Name:   "remove-recipient",
		Usage:  "remove a recipient of the encrypted trousseau",
		Action: RemoveRecipientAction,
		Flags: []libcli.Flag{
			VerboseFlag(),
		},
	}
}

func SetCommand() libcli.Command {
	return libcli.Command{
		Name:   "set",
		Usage:  "sets a key value pair in the store",
		Action: SetAction,
		Flags: []libcli.Flag{
			FileFlag(),
			VerboseFlag(),
		},
	}
}

func GetCommand() libcli.Command {
	return libcli.Command{
		Name:   "get",
		Usage:  "get a value from the trousseau",
		Action: GetAction,
		Flags: []libcli.Flag{
			FileFlag(),
		},
	}
}

func RenameCommand() libcli.Command {
	return libcli.Command{
		Name:   "rename",
		Usage:  "rename an existing key",
		Action: RenameAction,
		Flags: []libcli.Flag{
			OverwriteFlag(),
			VerboseFlag(),
		},
	}
}

func DelCommand() libcli.Command {
	return libcli.Command{
		Name:   "del",
		Usage:  "delete the point key pair from the store",
		Action: DelAction,
		Flags: []libcli.Flag{
			VerboseFlag(),
		},
	}
}

func KeysCommand() libcli.Command {
	return libcli.Command{
		Name:   "keys",
		Usage:  "Lists the store keys",
		Action: KeysAction,
		Flags: []libcli.Flag{
			VerboseFlag(),
		},
	}
}

func ShowCommand() libcli.Command {
	return libcli.Command{
		Name:   "show",
		Usage:  "shows trousseau content",
		Action: ShowAction,
	}
}

func MetaCommand() libcli.Command {
	return libcli.Command{
		Name:   "meta",
		Usage:  "shows trousseau metadata",
		Action: MetaAction,
	}
}

func UpgradeCommand() libcli.Command {
	return libcli.Command{
		Name: "upgrade",
		Usage: "Upgrade your data store to a newer version",
		Action: UpgradeAction,
		Flags: []libcli.Flag{
			YesFlag(),
			NoBackupFlag(),
		},
	}
}
