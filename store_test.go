package trousseau

import (
	"encoding/json"
	"github.com/stretchr/testify/assert"
	"io/ioutil"
	"os"
	"testing"
)

func TestEncodeStoreToFile_with_non_existing_path(t *testing.T) {
	non_existing_path := "/non/existing/file"
	store := make(Store)

	err := encodeStoreToFile(&store, non_existing_path)
	assert.Error(t, err)
}

func TestEncodeStoreToFile_with_existing_path(t *testing.T) {
	existing_path := "/tmp/file"
	defer os.Remove("/tmp/file")
	store := make(Store)

	err := encodeStoreToFile(&store, existing_path)
	assert.NoError(t, err)
	assert.True(t, pathExists("/tmp/file"))
}

func TestEncodeStoreToFile_with_fulfilled_store(t *testing.T) {
	existing_path := "/tmp/file"
	defer os.Remove("/tmp/file")
	store := make(Store)

	// let's fullfill that store
	store["abc"] = "123"
	store["easy as"] = "do re mi"

	// Write store to file
	err := encodeStoreToFile(&store, existing_path)
	assert.NoError(t, err)
	assert.True(t, pathExists("/tmp/file"))

	// Open store and decode the data to ensure key value pairs
	// have been set
	// Extract encoded store from file
	store_data, err := ioutil.ReadFile(existing_path)
	assert.NoError(t, err)

	decoded_store := make(Store)
	err = json.Unmarshal(store_data, &decoded_store)
	assert.NoError(t, err)
	assert.Equal(t, decoded_store["abc"], "123")
	assert.Equal(t, decoded_store["easy as"], "do re mi")
}

func TestDecodeStoreFromFile_with_non_existing_path(t *testing.T) {
	non_existing_path := "/non/existing/file"
	_, err := decodeStoreFromFile(non_existing_path)
	assert.Error(t, err)
}

func TestDecodeStoreFromFile_with_existing_path(t *testing.T) {
	existing_path := "/tmp/file"
	defer os.Remove("/tmp/file")

	// Create a store with data
	store := make(Store)
	store["abc"] = "123"
	store["easy as"] = "do re mi"

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(existing_path, encoded_store, 0764)
	assert.NoError(t, err)

	// Retrieve it's content using decodeStoreFromFile
	// and make sure data are what they should
	decoded_store, err := decodeStoreFromFile(existing_path)
	assert.NoError(t, err)
	assert.Equal(t, decoded_store["abc"], "123")
	assert.Equal(t, decoded_store["easy as"], "do re mi")
}

func TestCreateStoreFile(t *testing.T) {
	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	CreateStoreFile()
	defer os.Remove("/tmp/.trousseau")
	assert.True(t, pathExists("/tmp/.trousseau"))

	// Open store and decode the data to ensure an empty store
	// has been created
	store_data, err := ioutil.ReadFile("/tmp/.trousseau")
	assert.NoError(t, err)

	decoded_store := make(Store)
	err = json.Unmarshal(store_data, &decoded_store)
	assert.NoError(t, err)
}

func TestGetStoreKey_with_existing_key(t *testing.T) {
	trousseau_path := "/tmp/.trousseau"

	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	// Create a store with data
	store := make(Store)
	store["abc"] = "123"
	store["easy as"] = "do re mi"

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(trousseau_path, encoded_store, 0764)
	defer os.Remove(trousseau_path)
	assert.NoError(t, err)

	value, err := GetStoreKey("abc")
	assert.NoError(t, err)
	assert.Equal(t, value, "123")

	value, err = GetStoreKey("easy as")
	assert.NoError(t, err)
	assert.Equal(t, value, "do re mi")
}

func TestGetStoreKey_with_non_existing_key(t *testing.T) {
	trousseau_path := "/tmp/.trousseau"

	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	// Create a store with data
	store := make(Store)

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(trousseau_path, encoded_store, 0764)
	defer os.Remove(trousseau_path)
	assert.NoError(t, err)

	_, err = GetStoreKey("abc")
	assert.Error(t, err)
}

func TestSetStoreFile(t *testing.T) {
	trousseau_path := "/tmp/.trousseau"

	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	// Create a store without data
	store := make(Store)

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(trousseau_path, encoded_store, 0764)
	defer os.Remove(trousseau_path)
	assert.NoError(t, err)

	// Set some key-value pairs
	err = SetStoreKey("abc", "123")
	assert.NoError(t, err)

	err = SetStoreKey("easy as", "do re mi")
	assert.NoError(t, err)

	// Open store and decode the data to ensure keys have been set
	store_data, err := ioutil.ReadFile("/tmp/.trousseau")
	assert.NoError(t, err)

	decoded_store := make(Store)
	err = json.Unmarshal(store_data, &decoded_store)
	assert.NoError(t, err)

	assert.Equal(t, decoded_store["abc"], "123")
	assert.Equal(t, decoded_store["easy as"], "do re mi")
}

func TestDelStoreFile_with_existing_key(t *testing.T) {
	trousseau_path := "/tmp/.trousseau"

	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	// Create a store with a single key value pair
	store := make(Store)
	store["abc"] = "123"

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(trousseau_path, encoded_store, 0764)
	defer os.Remove(trousseau_path)
	assert.NoError(t, err)

	// Del the created key-value pair
	err = DelStoreKey("abc")
	assert.NoError(t, err)

	// Open store and decode the data to ensure keys have been set
	store_data, err := ioutil.ReadFile("/tmp/.trousseau")
	assert.NoError(t, err)

	decoded_store := make(Store)
	err = json.Unmarshal(store_data, &decoded_store)
	assert.NoError(t, err)

	_, ok := decoded_store["abc"]
	assert.False(t, ok)
}

func TestDelStoreFile_with_non_existing_key(t *testing.T) {
	trousseau_path := "/tmp/.trousseau"

	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	// Create a store without data
	store := make(Store)

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(trousseau_path, encoded_store, 0764)
	defer os.Remove(trousseau_path)
	assert.NoError(t, err)

	// Del the created key-value pair
	err = DelStoreKey("abc")
	assert.Error(t, err)
}

func TestListStoreKeys(t *testing.T) {
	trousseau_path := "/tmp/.trousseau"

	// Ensure store file is created in tmp
	current_home := os.Getenv("HOME")
	os.Setenv("HOME", "/tmp")
	defer os.Setenv("HOME", current_home)

	// Create a store with data
	store := make(Store)
	store["abc"] = "123"
	store["easy as"] = "do re mi"

	// Encode it
	encoded_store, err := json.Marshal(store)
	assert.NoError(t, err)

	// Write it to file
	err = ioutil.WriteFile(trousseau_path, encoded_store, 0764)
	defer os.Remove(trousseau_path)
	assert.NoError(t, err)

	// Open store and decode the data to ensure keys have been set
	store_data, err := ioutil.ReadFile("/tmp/.trousseau")
	assert.NoError(t, err)

	decoded_store := make(Store)
	err = json.Unmarshal(store_data, &decoded_store)
	assert.NoError(t, err)

	keys, err := ListStoreKeys()
	assert.NoError(t, err)
	assert.Equal(t, keys, []string{"abc", "easy as"})
}
