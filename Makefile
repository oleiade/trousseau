DEPS = $(go list -f '{{range .TestImports}}{{.}} {{end}}' ./...)

all:
	@(mkdir -p bin/)
	@(bash --norc -i ./scripts/build.sh)

cov:
	@(gocov test ./... | gocov-html > /tmp/coverage.html)
	@(open /tmp/coverage.html)

deps:
	@(go get github.com/kr/godep)

test: deps
	@(go list ./... | xargs -n1 go test)

.PNONY: all cov deps test

