package trousseau

type Store struct {
	Meta Meta    `json:"meta"`
	Data KVStore `json:"store"`
}

func NewStore(meta Meta) *Store {
	return &Store{
		Meta: meta,
		Data: KVStore{},
	}
}
