package openpgp

import "strings"

// isEmail checks if the provided string is
// of the email form. It does not intend to be robust
// email validation helper.
func isEmail(s string) bool {
	for _, symbol := range []string{"@", "."} {
		if !strings.Contains(s, symbol) {
			return false
		}
	}

	return true
}
