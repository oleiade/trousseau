# Base paths
ROOT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
TROUSSEAU_DIR := $(ROOT_DIR)/trousseau

# Trousseau version
VERSION=$(awk '/TROUSSEAU_VERSION/ { gsub("\"", ""); print $NF }' ${TROUSSEAU_DIR}/constants.go)

# Binaries paths
BIN_DIR = $(ROOT_DIR)/bin
TROUSSEAU_BIN = $(BIN_DIR)/trousseau

# Actions
DEPS = $(go list -f '{{range .TestImports}}{{.}} {{end}}' ./...)

all: deps trousseau 
	@(mkdir -p $(BIN_DIR))

deps:
	@(echo "-> Processing dependencies")
	@(go get github.com/kr/godep)
	@(go get ./...)

trousseau: deps
	@(echo "-> Compiling trousseau binary")
	@(mkdir -p $(BIN_DIR))
	@(go build -o $(TROUSSEAU_BIN)) 
	@(echo "-> trousseau binary created: $(TROUSSEAU_BIN)")

test: deps
	@(go list ./... | xargs -n1 go test)

format:
	@(go fmt ./...)
	@(go vet ./...)

.PNONY: all deps trousseau test format 

