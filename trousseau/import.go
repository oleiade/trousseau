package trousseau

import (
	"fmt"
	"github.com/codegangsta/cli"
)

type ImportStrategy uint32

// ImportStore imports the src encrypted data store content
// into dest data store, respecting the provided import strategy.
func ImportStore(src, dest *Store, strategy ImportStrategy) error {
	switch strategy {
	case IMPORT_YOURS:
		for key, value := range src.Data.Container {
			if _, ok := dest.Data.Container[key]; !ok {
				dest.Data.Container[key] = value
			}
		}
	case IMPORT_THEIRS:
		for key, value := range src.Data.Container {
			dest.Data.Container[key] = value
		}
	case IMPORT_OVERWRITE:
		dest.Data.Container = src.Data.Container
	}

	return nil
}

func (s *ImportStrategy) FromCliContext(c *cli.Context) error {
	var yours bool = c.Bool("yours")
	var theirs bool = c.Bool("theirs")
	var overwrite bool = c.Bool("overwrite")
	activated := 0

	// Ensure two import strategies were not provided at
	// the same time. Otherwise, throw an error
	for _, flag := range []bool{yours, theirs, overwrite} {
		if flag {
			activated += 1
		}
		if activated >= 2 {
			return fmt.Errorf("--yours, --theirs and --overwrite options are mutually exclusive")
		}
	}

	// Return proper ImportStrategy according to
	// provided flags
	if overwrite == true {
		*s = IMPORT_OVERWRITE
	} else if theirs == true {
		*s = IMPORT_THEIRS
	} else {
		*s = IMPORT_YOURS
	}

	return nil
}
