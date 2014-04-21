package trousseau

import (
	"fmt"
	"github.com/Sirupsen/logrus"
)

var Logger = logrus.New()

type RawFormatter struct{}

func (f *RawFormatter) Format(entry *logrus.Entry) ([]byte, error) {
	var serialized []byte
	var msg []byte = []byte(entry.Data["msg"].(string))
	serialized = append(serialized, msg...)

	return append(serialized, '\n'), nil
}

func (f *RawFormatter) AppendKeyValue(serialized []byte, key, value interface{}) []byte {
	if _, ok := value.(string); ok {
		return append(serialized, []byte(fmt.Sprintf("%v='%v' ", key, value))...)
	} else {
		return append(serialized, []byte(fmt.Sprintf("%v=%v ", key, value))...)
	}
}
