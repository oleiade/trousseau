// Shows example use of the keyring package
//
// May need to be built with a platform-specific build flag to specify a
// provider. See keyring documentation for details.
//
package main

import (
	"os"
	"fmt"
	"code.google.com/p/gopass"
	"github.com/tmc/keyring"
)

func main() {
	if pw, err := keyring.Get("keyring_example", "jack"); err == nil {
		fmt.Println("current stored password:", pw)
	} else if err == keyring.ErrNotFound {
		fmt.Println("no password stored yet")
	} else {
		fmt.Println("got unexpected error:", err)
		os.Exit(1)
	}
	pw, err := gopass.GetPass("enter new password: ")
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	fmt.Println("setting keyring_example/jack to..", pw)
	err = keyring.Set("keyring_example", "jack", pw)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	fmt.Println("fetching keyring_example/jack..")
	if pw, err := keyring.Get("keyring_example", "jack"); err == nil {
		fmt.Println("got", pw)
	} else {
		fmt.Println("error:", err)
	}
}
