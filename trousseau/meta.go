package trousseau

import (
	"errors"
	"fmt"
	"time"
)

type Meta struct {
	CreatedAt        string   `json:"created_at"`
	LastModifiedAt   string   `json:"last_modified_at"`
	Recipients       []string `json:"recipients"`
	TrousseauVersion string   `json:"version"`
}

func (m *Meta) updateLastModificationMarker() {
	m.LastModifiedAt = time.Now().String()
}

func (m *Meta) containsRecipient(recipient string) (status bool, index int) {
	for index, r := range m.Recipients {
		if r == recipient {
			return true, index
		}
	}

	return false, -1
}

func (m *Meta) AddRecipient(recipient string) error {
	in, _ := m.containsRecipient(recipient)
	if in {
		errMsg := fmt.Sprintf("Recipient %s already mapped to store metadata", recipient)
		return errors.New(errMsg)
	} else {
		m.Recipients = append(m.Recipients, recipient)
	}

	return nil
}

func (m *Meta) RemoveRecipient(recipient string) error {
	in, idx := m.containsRecipient(recipient)
	if !in {
		errMsg := fmt.Sprintf("Recipient %s not mapped in store metadata", recipient)
		return errors.New(errMsg)
	} else {
		newRecipients := make([]string, len(m.Recipients)-1)
		copy(newRecipients[0:idx], m.Recipients[0:idx])
		copy(newRecipients[:idx], m.Recipients[:idx+1])
		m.Recipients = newRecipients
	}

	return nil

}
