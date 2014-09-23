# Base paths
ROOT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

# Trousseau version
VERSION=$(awk '/TROUSSEAU_VERSION/ { gsub("\"", ""); print $NF }' ${ROOT_DIR}/constants.go)

# Commands paths
CMD_DIR := $(ROOT_DIR)/cmd
TROUSSEAU_CMD_DIR = $(CMD_DIR)/trousseau

# Binaries paths
BIN_DIR = $(ROOT_DIR)/bin
TROUSSEAU_BIN = $(BIN_DIR)/trousseau

# Third party binaries paths
BATS_BIN ?= $(shell which bats)

# Integration tests resources
INTEGRATION_TEST_DIR := $(ROOT_DIR)/tests
INTEGRATION_TEST_FILES := $(wildcard $(INTEGRATION_TEST_DIR)/*.bats)
INTEGRATION_TEST_FILES := $(filter-out $(INTEGRATION_TEST_DIR)/auth.bats, $(INTEGRATION_TEST_FILES))

# Actions
DEPS = $(go list -f '{{range .TestImports}}{{.}} {{end}}' ./...)

all: deps trousseau 
	@(mkdir -p $(BIN_DIR))

deps:
	@(echo "-> Processing dependencies")
	@(go get github.com/kr/godep)
	@(godep restore)

trousseau: deps
	@(echo "-> Compiling trousseau binary")
	@(mkdir -p $(BIN_DIR))
	@(cd $(TROUSSEAU_CMD_DIR) && go build -o $(TROUSSEAU_BIN)) 
	@(echo "-> trousseau binary created: $(TROUSSEAU_BIN)")

test: unit integration

unit: deps
	@(go list ./... | xargs -n1 go test)

# Running integration depends on bats test framework
# https://github.com/sstephenson/bats
# Make sure to set $BATS_BIN variable to point
# to bats eecutable via 'env BATS_BIN=myexec make integration'
integration: deps
	@(${BATS_BIN} $(INTEGRATION_TEST_FILES))

format:
	@(go fmt ./...)
	@(go vet ./...)

.PNONY: all deps trousseau test format 

