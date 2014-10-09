package trousseau

import (
	"fmt"
	"github.com/howeyc/gopass"
)

// PromptForHiddenInput will prompt on stdin with the provided
// message and will hide the user input. This is intended to be used
// to ask the user for password or passphrase
func PromptForHiddenInput(msg string) string {
	fmt.Printf(msg)
	return string(gopass.GetPasswd())
}
