#
# VARIABLES 
#

TROUSSEAU_VERSION 	= `awk '/TROUSSEAU_VERSION/ { gsub("\"", ""); print $$NF }' $(CURDIR)/constants.go`

#
# Golang packages definition
#
PROJECT_PACKAGE 	= github.com/oleiade/trousseau
TROUSSEAU_PACKAGE 	= github.com/oleiade/trousseau/trousseau

#
# Directories
#
GOPATH_DIR 			= $(CURDIR)/.gopath
SRC_DIR 			= $(GOPATH_DIR)/src
BIN_DIR 			= $(CURDIR)/bin
DIST_DIR 			= $(CURDIR)/dist

PROJECT_DIR 		= $(SRC_DIR)/$(PROJECT_PACKAGE)
TROUSSEAU_DIR 		= $(SRC_DIR)/$(TROUSSEAU_PACKAGE)

#
# Executables definition
#
TROUSSEAU_BIN = $(BIN_DIR)/trousseau

#
# GOPATH definition
#
GOPATH ?= $(GOPATH_DIR)
export GOPATH

#
# Compilations options
#
GO_OPTIONS 			= -a 

#
# Packaging options
#
XC_ARCH 			?= "386 amd64 arm"
XC_OS 				?= "linux darwin windows freebsd openbsd"


#
# Compilation rules
#
all: $(TROUSSEAU_BIN)

$(TROUSSEAU_BIN): $(TROUSSEAU_DIR)
				@mkdir -p $(dir $@)
				@cd $<; go build -o $@
				@echo $@ created.

$(PROJECT_DIR):
				@mkdir -p $(dir $@)
				@if [ -h $@ ]; then rm -f $@; fi; ln -sf $(CURDIR)/ $@

$(TROUSSEAU_DIR): $(PROJECT_DIR)
				@cd $<; go get -d

clean:
	@rm -rf $(dir $(TROUSSEAU_BIN))
ifeq ($(GOPATH), $(BUILD_DIR))
	  @rm -rf $(BUILD_DIR)
else ifneq ($(PROJECT_DIR), $(realpath $(PROJECT_DIR)))
	  @rm -f $(PROJECT_DIR)
endif

package:
	@(echo $(TROUSSEAU_VERSION))
	@(go get github.com/laher/goxc)
	@(cd $(TROUSSEAU_DIR); goxc -arch=$(XC_ARCH) -os=$(XC_OS) -pv=$(TROUSSEAU_VERSION) -d=$(DIST_DIR) -tasks-="go-test")

.PHONY: all clean $(TROUSSEAU_BIN) $(TROUSSEAU_DIR) $(PROJECT_DIR)

