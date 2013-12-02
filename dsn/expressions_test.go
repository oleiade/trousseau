package dsn

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestNamedExpression(t *testing.T) {
	assert.Equal(t, NamedExpression("abc", "[123]"), "(?P<abc>[123])")
}

func TestOrExpressions(t *testing.T) {
	assert.Equal(t, OrExpressions("[a-z]", "[0-9]"), "[a-z]|[0-9]")
}

func TestNamedOrExpressions(t *testing.T) {
	orExp := OrExpressions("[a-z]", "[0-9]")
	namedExp := NamedExpression("test", orExp)

	assert.Equal(t, namedExp, "(?P<test>[a-z]|[0-9])")
}
