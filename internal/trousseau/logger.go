package trousseau

import (
	"log"
	"os"
)

var InfoLogger = log.New(os.Stdout, "", 0)
var ErrorLogger = log.New(os.Stderr, "Error: ", 0)
