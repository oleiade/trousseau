// +build gnome_keyring

package keyring

/*
#cgo pkg-config: gnome-keyring-1 glib-2.0
#include <stdlib.h>
#include "gnome-keyring.h"

GnomeKeyringPasswordSchema keyring_schema =
  { GNOME_KEYRING_ITEM_GENERIC_SECRET,
    { { "username", GNOME_KEYRING_ATTRIBUTE_TYPE_STRING },
      { "service",  GNOME_KEYRING_ATTRIBUTE_TYPE_STRING },
      { NULL,      0 } } };

// wrap the gnome calls because cgo can't deal with vararg functions

GnomeKeyringResult gkr_set_password(gchar *description, gchar *service, gchar *username, gchar *password) {
	return gnome_keyring_store_password_sync(
		&keyring_schema,
		NULL,
		description,
		password,
		"service", service,
		"username", username,
		NULL);
}

GnomeKeyringResult gkr_get_password(gchar *service, gchar *username, char **password) {
	return gnome_keyring_find_password_sync(
		&keyring_schema,
		(gchar**)password,
		"service", service,
		"username", username,
		NULL);
}
*/
import "C"

import (
	"errors"
	"fmt"
	"unsafe"
)

var errorMap = map[C.GnomeKeyringResult]error{
	C.GNOME_KEYRING_RESULT_IO_ERROR:  errors.New("Error communicating with the gnome-keyring daemon"),
	C.GNOME_KEYRING_RESULT_CANCELLED: errors.New("Operation was cancelled"),
	C.GNOME_KEYRING_RESULT_NO_MATCH:  ErrNotFound,
}

type gnomeKeyring struct{}

func (p gnomeKeyring) Set(Service, Username, Password string) error {
	desc := (*C.gchar)(C.CString("Username and password for " + Service))
	username := (*C.gchar)(C.CString(Username))
	service := (*C.gchar)(C.CString(Service))
	password := (*C.gchar)(C.CString(Password))
	defer C.free(unsafe.Pointer(desc))
	defer C.free(unsafe.Pointer(username))
	defer C.free(unsafe.Pointer(service))
	defer C.free(unsafe.Pointer(password))

	result := C.gkr_set_password(desc, service, username, password)

	if result != C.GNOME_KEYRING_RESULT_OK {
		if knownErr, ok := errorMap[result]; ok {
			return knownErr
		}
		return fmt.Errorf("Unknown gnome-keyring error: %d", int(result))
	}
	return nil
}

func (p gnomeKeyring) Get(Service string, Username string) (string, error) {
	var pw *C.char
	username := (*C.gchar)(C.CString(Username))
	service := (*C.gchar)(C.CString(Service))
	defer C.free(unsafe.Pointer(username))
	defer C.free(unsafe.Pointer(service))

	result := C.gkr_get_password(service, username, &pw)
	defer C.gnome_keyring_free_password((*C.gchar)(pw))

	if result != C.GNOME_KEYRING_RESULT_OK {
		if err, ok := errorMap[result]; ok {
			return "", err
		}
		return "", fmt.Errorf("Unknown gnome-keyring error: %d", int(result))
	}
	return C.GoString(pw), nil
}

func init() {
	defaultProvider = gnomeKeyring{}
}
