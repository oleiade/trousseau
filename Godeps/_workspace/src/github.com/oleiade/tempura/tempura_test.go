package tempura

import (
	"fmt"
	"io/ioutil"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func ExampleFromBytes() {
	var tmp *TempFile
	var err error

	// Creates a temporary file in /tmp dir, prefixed with "test_"
	// and containing three bytes "a", "b", "c"
	tmp, err = FromBytes("/tmp", "test_", []byte{'a', 'b', 'c'})
	if err != nil {
		// Handle err
	}
	defer tmp.Close()
	defer os.Remove(tmp.Name())

	// The tmp file descriptor is seeked to 0 and ready to be read/written
	data, err := tmp.Read(make([]byte, 3))
	if err != nil {
		// Handle error
	}

	fmt.Println(string(data))
	// Output: "abc"
}

func TestFromBytes_creates_a_file(t *testing.T) {
	tmp, err := FromBytes("/tmp", "test_tempura_", []byte{'a', 'b', 'c'})
	defer os.Remove(tmp.Name())
	assert.NoError(t, err)

	_, err = os.Stat(tmp.Name())
	assert.NoError(t, err)
}

func TestFromBytes_returns_an_opened_fd(t *testing.T) {
	tmp, _ := FromBytes("/tmp", "test_tempura_", []byte{'a', 'b', 'c'})
	defer os.Remove(tmp.Name())

	_, err := tmp.Read([]byte{})
	assert.NoError(t, err)
}

func TestFromBytes_returns_an_opened_seeked_to_zero_fd(t *testing.T) {
	input := []byte{'a', 'b', 'c'}
	output := make([]byte, 1)

	tmp, _ := FromBytes("/tmp", "test_tempura_", input)
	defer os.Remove(tmp.Name())
	n, err := tmp.Read(output)

	assert.NoError(t, err)
	assert.Equal(t, n, 1)
	assert.Equal(t, output, []byte{'a'})
}

func TestCreate_returns_a_valid_path(t *testing.T) {
	p, err := Create("/tmp", "test_tempura_", []byte{'a', 'b', 'c'})
	assert.NoError(t, err)

	_, err = os.Stat(p)
	assert.NoError(t, err)

	data, err := ioutil.ReadFile(p)
	assert.NoError(t, err)
	assert.Equal(t, data, []byte{'a', 'b', 'c'})
}
