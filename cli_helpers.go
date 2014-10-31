package trousseau

import (
	"fmt"
	"os"

	"github.com/howeyc/gopass"
)

// PromptForHiddenInput will prompt on stdin with the provided
// message and will hide the user input. This is intended to be used
// to ask the user for password or passphrase
func PromptForHiddenInput(msg string) string {
	fmt.Printf(msg)
	return string(gopass.GetPasswd())
}

func PromptForHiddenInputConfirm() string {
	i := 0
	for {
		if i >= 3 {
			fmt.Println("Error occurred. Exiting...")
			os.Exit(1)
		}

		fmt.Printf("Passphrase: ")
		pass1 := string(gopass.GetPasswd())

		if pass1 == "" {
			fmt.Printf("Empty passphrase not allowed\n\n")
			i++
			continue
		}

		fmt.Printf("Confirm passphrase: ")

		pass2 := string(gopass.GetPasswd())
		if pass1 == pass2 {
			return pass1
		} else {
			fmt.Printf("Passphrases did not match. Please try again\n\n")
		}
		i++
	}
}
