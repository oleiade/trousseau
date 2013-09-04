SYS_OS := $(shell uname -s)

TROUSSEAU_PACKAGE := bitbucket.com/facteur/trousseau
BUILD_SRC := build_src
BUILD_PATH := ${BUILD_SRC}/src/${TROUSSEAU_PACKAGE}

GIT_ROOT := $(shell git rev-parse --show-toplevel)
BUILD_DIR := $(CURDIR)/.gopath

GIT_COMMIT = $(shell git rev-parse --short HEAD)
GIT_STATUS = $(shell test -n "`git status --porcelain`" && echo "+CHANGES")

GOPATH ?= $(BUILD_DIR)
export GOPATH

GO_OPTIONS ?= -a -ldflags=$(LDFLAGS)
ifeq ($(VERBOSE), 1)
GO_OPTIONS += -v
endif

BUILD_OPTIONS = -a -ldflags "-X main.GITCOMMIT $(GIT_COMMIT)$(GIT_STATUS)"
COMPILATION_OPTIONS = CGO_ENABLED=0
LDFLAGS := '-w -d'

SRC_DIR := $(GOPATH)/src

TROUSSEAU_DIR := $(SRC_DIR)/$(TROUSSEAU_PACKAGE)
TROUSSEAU_MAIN := $(TROUSSEAU_DIR)/trousseau

TROUSSEAU_BIN_RELATIVE := bin/trousseau
TROUSSEAU_BIN := $(CURDIR)/$(TROUSSEAU_BIN_RELATIVE)

.PHONY: all clean test hack $(TROUSSEAU_BIN) $(TROUSSEAU_DIR)

all: $(TROUSSEAU_BIN)

$(TROUSSEAU_BIN): $(TROUSSEAU_DIR)
				@mkdir -p $(dir $@)
				@(cd $(TROUSSEAU_MAIN); $(COMPILATION_OPTIONS) go build $(GO_OPTIONS) $(BUILD_OPTIONS) -o $@)
				@echo $(TROUSSEAU_BIN_RELATIVE) is created.

$(TROUSSEAU_DIR):
				@mkdir -p $(dir $@)
				@if [ -h $@ ]; then rm -f $@; fi; ln -sf $(CURDIR)/ $@
				@(cd $(TROUSSEAU_MAIN); go get -d $(GO_OPTIONS))

clean:
	@rm -rf $(dir $(TROUSSEAU_BIN))
ifeq ($(GOPATH), $(BUILD_DIR))
		@rm -rf $(BUILD_DIR)
else ifneq ($(TROUSSEAU_DIR), $(realpath $(TROUSSEAU_DIR)))
	@rm -f $(TROUSSEAU_DIR)
endif

test:
	@(go get "github.com/stretchr/testify/assert")
	@(cd $(TROUSSEAU_DIR); sudo -E go test -v $(GO_OPTIONS))

fmt:
	@gofmt -s -l -w .

