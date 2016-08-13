package keyring

import "errors"

var (
	// ErrNotFound means the requested password was not found
	ErrNotFound = errors.New("keyring: Password not found")
	// ErrNoDefault means that no default keyring provider has been found
	ErrNoDefault = errors.New("keyring: No suitable keyring provider found (check your build flags)")

	defaultProvider   provider
	providerInitError error
)

// provider provides a simple interface to keychain sevice
type provider interface {
	Get(service, username string) (string, error)
	Set(service, username, password string) error
}

// Get gets the password for a paricular Service and Username using the
// default keyring provider.
func Get(service, username string) (string, error) {
	if providerInitError != nil {
		return "", providerInitError
	} else if defaultProvider == nil {
		return "", ErrNoDefault
	}

	return defaultProvider.Get(service, username)
}

// Set sets the password for a particular Service and Username using the
// default keyring provider.
func Set(service, username, password string) error {
	if providerInitError != nil {
		return providerInitError
	} else if defaultProvider == nil {
		return ErrNoDefault
	}

	return defaultProvider.Set(service, username, password)
}
