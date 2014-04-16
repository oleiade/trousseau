package trousseau

import (
	"os"
)

// exists returns whether the given file or directory exists or not
func pathExists(path string) bool {
	_, err := os.Stat(path)
	if err == nil {
		return true
	}

	if os.IsNotExist(err) {
		return false
	}

	return false
}
