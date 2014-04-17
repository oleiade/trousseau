package keyring_test

import (
	"fmt"
	"github.com/tmc/keyring"
)

func ExampleGet() {
	keyring.Set("keyring-test", "jack", "test password")
	pw, _ := keyring.Get("keyring-test", "jack")
	fmt.Println("pw:", pw)
	// don't ignore errors like this in your code
	// Output:
	// pw: test password
}
