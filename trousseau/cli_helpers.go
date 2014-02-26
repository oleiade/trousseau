package trousseau

import (
	"fmt"
	"github.com/howeyc/gopass"
)

func PromptForPassword() string {
	fmt.Printf("Password: ")
	return string(gopass.GetPasswd())
}
