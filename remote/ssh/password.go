package ssh

type password string

func (p password) Password(_ string) (string, error) {
	return string(p), nil
}
