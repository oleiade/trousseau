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
BATS_BIN := $(shell which bats 2>/dev/null)
GOXC_BIN := $(shell which goxc 2>/dev/null)

# Integration tests resources
INTEGRATION_TEST_DIR := $(ROOT_DIR)/tests
INTEGRATION_TEST_FILES := $(wildcard $(INTEGRATION_TEST_DIR)/*.bats)
INTEGRATION_TEST_FILES := $(filter-out $(INTEGRATION_TEST_DIR)/auth.bats, $(INTEGRATION_TEST_FILES))

# Actions
DEPS = $(go list -f '{{range .TestImports}}{{.}} {{end}}' ./...)

all: trousseau
	@(mkdir -p $(BIN_DIR))

trousseau:
	@(go get github.com/kr/godep)
	@(echo "-> Compiling trousseau binary")
	@(mkdir -p $(BIN_DIR))
	@(cd $(TROUSSEAU_CMD_DIR) && godep go build -o $(TROUSSEAU_BIN)) 
	@(echo "-> trousseau binary created: $(TROUSSEAU_BIN)")

test: unit integration

unit:
	@(go list ./... | xargs -n1 go test)

# Running integration depends on bats test framework
# https://github.com/sstephenson/bats
# Make sure to set $BATS_BIN variable to point
# to bats eecutable via 'env BATS_BIN=myexec make integration'
integration: all
ifdef BATS_BIN
	@(${BATS_BIN} $(INTEGRATION_TEST_FILES))
else
	@(echo "bats was not found on your PATH. Unable to run integration tests.")
endif

# package rule will build debian and osx packages using the goxc (https://github.com/laher/goxc)
# tool. Before running this command, make sure to call "goxc -t" before moving on with the package
# command.
package:
ifdef GOXC_BIN
	@(goxc -c $(ROOT_DIR)/.goxc.json)
else
	@(echo "goxc was not found on your PATH. Unable to run integration tests.")
endif

format:
	@(go fmt ./...)
	@(go vet ./...)

.PHONY: all trousseau test unit integration package format

