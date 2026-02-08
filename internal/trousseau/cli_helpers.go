package trousseau

import (
	"fmt"
	"os"

	"golang.org/x/term"
)

const passPhraseMsg string = "\rPassphrase: "
const confirmMsg string = "\rConfirm Passphrase: "
const errorMsg string = "\rPassphrase error occurred. Exiting..."

// PromptForHiddenInput will prompt on stdin with the provided
// message and will hide the user input. This is intended to be used
// to ask the user for password or passphrase
func PromptForHiddenInput(msg string) string {
	fmt.Print(msg)
	pass, _ := term.ReadPassword(int(os.Stdin.Fd()))

	return string(pass)
}

func PromptForHiddenInputConfirm() string {
	i := 0
	for {
		if i >= 3 {
			fmt.Println(errorMsg)
			os.Exit(1)
		}

		fmt.Print(passPhraseMsg)
		inputPass, _ := term.ReadPassword(int(os.Stdin.Fd()))

		if string(inputPass) == "" {
			fmt.Printf("Empty passphrase not allowed\n\n")
			i++
			continue
		}

		fmt.Print(confirmMsg)

		confirmPass, _ := term.ReadPassword(int(os.Stdin.Fd()))
		if string(inputPass) == string(confirmPass) {
			return string(inputPass)
		} else {
			fmt.Printf("Passphrases did not match. Please try again\n\n")
		}
		i++
	}
}
