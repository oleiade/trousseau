package remote

import (
    "os"
    "path/filepath"
)


// Global data store file path
var gStorePath string = filepath.Join(os.Getenv("HOME"), STORE_FILENAME)
