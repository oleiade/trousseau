package remote

import "io"

// Handler is an interface exposing methods upload
// and download data to and from a remote storage service.
type Handler interface {
	Pull(dest string, r io.ReadSeeker) error
	Push(src string, w io.Writer) error
}
