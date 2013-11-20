#
# VARIABLES 
#


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

.PHONY: all clean $(TROUSSEAU_BIN) $(TROUSSEAU_DIR) $(PROJECT_DIR)

