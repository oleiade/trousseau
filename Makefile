#
# Variables
#
TROUSSEAU_VERSION = `awk '/TROUSSEAU_VERSION/ { gsub("\"", ""); print $$NF }' $(CURDIR)/constants.go`

#
# Golang packages definition
#
PROJECT_PACKAGE   = github.com/oleiade/trousseau
TROUSSEAU_PACKAGE = $(PROJECT_PACKAGE)/trousseau
DSN_PACKAGE 	  = $(PROJECT_PACKAGE)/dsn

#
# Directories
#
GOPATH_DIR = $(CURDIR)/.gopath
SRC_DIR    = $(GOPATH_DIR)/src
BIN_DIR    = $(CURDIR)/bin
DIST_DIR   = $(CURDIR)/dist

PROJECT_DIR   = $(SRC_DIR)/$(PROJECT_PACKAGE)
TROUSSEAU_DIR = $(SRC_DIR)/$(TROUSSEAU_PACKAGE)
DSN_DIR 	  = $(SRC_DIR)/$(DSN_PACKAGE)

#
# Executables definition
#
TROUSSEAU_BIN_NAME = trousseau
TROUSSEAU_BIN = $(BIN_DIR)/$(TROUSSEAU_BIN_NAME)

#
# GOPATH definition
#
GOPATH     ?= $(GOPATH_DIR)
export GOPATH
OLD_GOPATH  = $(GOPATH)

#
# Compilations options
#
GO_OPTIONS = -a

#
# Packaging options
#
XC_ARCH ?= "386 amd64 arm"
XC_OS   ?= "linux darwin windows freebsd openbsd"

#
# Compilation rules
#
all: $(TROUSSEAU_BIN)

$(TROUSSEAU_BIN): $(TROUSSEAU_DIR)
				@mkdir -p $(dir $@)
				@export GOPATH=$(GOPATH_DIR); \
				 cd $<; go build -o $@; \
				 export GOPATH=$(OLD_GOPATH);

$(PROJECT_DIR):
				@mkdir -p $(GOPATH_DIR)

$(TROUSSEAU_DIR): $(PROJECT_DIR)
				@export GOPATH=$(GOPATH_DIR); \
				 cd $(CURDIR)/trousseau/; go get -d; \
				 export GOPATH=$(OLD_GOPATH);

clean:
				@rm -rf $(dir $(TROUSSEAU_BIN))
ifeq ($(GOPATH), $(BUILD_DIR))
				@rm -rf $(BUILD_DIR)
else ifneq ($(PROJECT_DIR), $(realpath $(PROJECT_DIR)))
				@rm -f $(PROJECT_DIR)
endif

test:
				@(go test -i)
				@(echo "--- Testing trousseau package ---")
				@(go test -v)
				@(echo "--- Testing dsn package ---")
				@(cd $(DSN_DIR); go test -v; cd -)

package:
				@(echo $(TROUSSEAU_VERSION))
				@(go get github.com/laher/goxc)
				@(cd $(TROUSSEAU_DIR); goxc -arch=$(XC_ARCH) -os=$(XC_OS) -pv=$(TROUSSEAU_VERSION) -d=$(DIST_DIR) -tasks-="go-test")

.PHONY: all clean $(TROUSSEAU_BIN) $(TROUSSEAU_DIR) $(PROJECT_DIR)
