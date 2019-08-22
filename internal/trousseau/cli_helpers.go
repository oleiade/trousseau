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
	pass, _ := gopass.GetPasswd()

	return string(pass)
}

func PromptForHiddenInputConfirm() string {
	i := 0
	for {
		if i >= 3 {
			fmt.Println(errorMsg)
			os.Exit(1)
		}

		fmt.Printf(passPhraseMsg)
		inputPass, _ := gopass.GetPasswd()

		if string(inputPass) == "" {
			fmt.Printf("Empty passphrase not allowed\n\n")
			i++
			continue
		}

		fmt.Printf(confirmMsg)

		confirmPass, _ := gopass.GetPasswd()
		if string(inputPass) == string(confirmPass) {
			return string(inputPass)
		} else {
			fmt.Printf("Passphrases did not match. Please try again\n\n")
		}
		i++
	}
}
