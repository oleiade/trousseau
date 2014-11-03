package trousseau

import (
	"fmt"
	"os"

	"github.com/howeyc/gopass"
)

const passPhraseMsg string = "Passphrase: "
const confirmMsg string = "Confirm Passphrase: "
const errorMsg string = "Passphrase error occurred. Exiting..."

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
			fmt.Println(errorMsg)
			os.Exit(1)
		}

		fmt.Printf(passPhraseMsg)
		inputPass := string(gopass.GetPasswd())

		if inputPass == "" {
			fmt.Printf("Empty passphrase not allowed\n\n")
			i++
			continue
		}

		fmt.Printf(confirmMsg)

		confirmPass := string(gopass.GetPasswd())
		if inputPass == confirmPass {
			return inputPass
		} else {
			fmt.Printf("Passphrases did not match. Please try again\n\n")
		}
		i++
	}
}
