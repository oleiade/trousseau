#!/bin/bash
#
# This script builds the application from source.
set -e

# Get the parent directory of where this script is.
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ] ; do SOURCE="$(readlink "$SOURCE")"; done
DIR="$( cd -P "$( dirname "$SOURCE" )/.." && pwd )"

# Get the trousseau package dir
TROUSSEAU_DIR="${DIR}/trousseau"

# Change into that directory
cd $DIR

# Get the version
VERSION=$(awk '/TROUSSEAU_VERSION/ { gsub("\"", ""); print $NF }' ${TROUSSEAU_DIR}/constants.go)

# Install dependencies
echo "--> Installing dependencies"
go get ./...

# Build!
echo "--> Building"
godep go build -ldflags "-X main.VERSION ${VERSION}" -o bin/trousseau
echo "bin/trousseau${EXTENSION} created"

# Copy binary to gopath
cp bin/trousseau${EXTENSION} $GOPATH/bin

