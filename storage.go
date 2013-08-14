package trousseau

// RemoteStorage is an interface exposing methods to upload
// trousseau file to a remote based location
type RemoteStorage interface {
	Connect()
	Push(string) error
	Pull(string) error
}
