package dsn

import (
	"fmt"
	"strings"
)

const (
	SCHEME_REGEXP = "[a-z0-9]+"
	ID_REGEXP     = "[^:]+"
	SECRET_REGEXP = "[^@]+"
	HOST_REGEXP   = "[a-zA-Z0-9_.-]+"
	PORT_REGEXP   = "[a-zA-Z0-9-]+"
	PATH_REGEXP   = "[a-zA-Z0-9/_.-]+"
	BUCKET_REGEXP = "[a-z-.]{3,63}"
	REGION_REGEXP = "[a-zA-Z]*-[a-zA-Z]*-[0-9]"
)

func NamedExpression(name string, expression string) string {
	return fmt.Sprintf("(?P<%s>%s)", name, expression)
}

func OrExpressions(expressions ...string) string {
	return strings.Join(expressions, "|")
}
