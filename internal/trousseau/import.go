package trousseau

type ImportStrategy uint32

// Import strategies enumeration
const (
	IMPORT_YOURS     = 0x0
	IMPORT_THEIRS    = 0x1
	IMPORT_OVERWRITE = 0x2
)

// ImportStore imports the src encrypted data store content
// into dest data store, respecting the provided import strategy.
func ImportStore(src, dest *SecretStore, strategy ImportStrategy) error {
	switch strategy {
	case IMPORT_YOURS:
		for key, value := range src.Data {
			if _, ok := dest.Data[key]; !ok {
				dest.Data[key] = value
			}
		}
	case IMPORT_THEIRS:
		for key, value := range src.Data {
			dest.Data[key] = value
		}
	case IMPORT_OVERWRITE:
		dest.Data = src.Data
	}

	return nil
}
