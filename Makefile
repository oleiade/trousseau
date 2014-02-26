DEPS = $(go list -f '{{range .TestImports}}{{.}} {{end}}' ./...)

all: deps
	@(mkdir -p bin/)
	@(bash --norc -i ./scripts/build.sh)

cov:
	@(gocov test ./... | gocov-html > /tmp/coverage.html)
	@(open /tmp/coverage.html)

deps:
	@(go get -d -v ./...)
	@(echo $(DEPS) | xargs -n1 go get -d)

test: deps
	@(go list ./... | xargs -n1 go test)

.PNONY: all cov deps test

