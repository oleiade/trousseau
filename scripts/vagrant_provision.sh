#!/bin/bash

# Define base go environment variables
GOROOT=/usr/local/go
GOPATH=/usr/local/gopath

# Install necessary system packages
echo "---> installing system packages"
sudo apt-get update -qq -y > /dev/null
sudo apt-get install -qq -y git wget vim nano build-essential pkg-config mercurial subversion > /dev/null
echo "<--- done"

# If go environment is not already provisioned on the box
# Then download and install go, and setup properly GOROOT
# and GOPATH dirs. This step takes care of adding the go environement
# variables in system /etc/environement so user has nothing more to
# do once it logs in.
if [ ! -d $GOROOT ]; then
	echo "---> installing go lang tools"
	wget --quiet https://storage.googleapis.com/golang/go1.3.3.linux-amd64.tar.gz > /dev/null
	sudo tar -C /usr/local/ -xzf go1.3.3.linux-amd64.tar.gz > /dev/null

	sudo echo "export GOROOT=$GOROOT" >> /etc/environment
	sudo mkdir -p /usr/local/gopath/{src,bin,pkg} && echo "export GOPATH=$GOPATH" >> /etc/environment
	sudo echo "export PATH=$PATH:$GOROOT/bin" >> /etc/environment 
	sudo echo "export PATH=$PATH:$GOPATH/bin" >> /etc/environment

	rm -rf go1.3.3.linux-amd64.tar.gz > /dev/null
	echo "<--- done"
fi

# Installing test tools, and adding them to system path
echo "---> running tests"
git clone --branch v0.4.0 https://github.com/sstephenson/bats.git /tmp/bats
cd /tmp/bats
./install.sh /usr/local
echo "<-- done"

echo "---> installing the test gpg keys"
gpg --quiet --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_public_first_test.key &> /dev/null
gpg --quiet --allow-secret-key-import --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_private_first_test.key &> /dev/null
gpg --quiet --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_public_second_test.key &> /dev/null
gpg --quiet --allow-secret-key-import --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_private_second_test.key &> /dev/null
echo "<--- done"

echo "---> building trousseau"
export GOPATH=$GOPATH
export PATH=$PATH:$GOROOT/bin
cd $GOPATH/src/github.com/oleiade/trousseau && make
echo "<--- done"

echo "---> running tests"
cd $GOPATH/src/github.com/oleiade/trousseau/ && make test
echo "<-- done"

echo "---> configuring vagrant user"
echo "cd $GOPATH/src/github.com/oleiade/trousseau" >> ~vagrant/.bash_profile
echo "<--- done"

echo "---> running tests"
# Install bats framework
git clone --branch v0.4.0 https://github.com/sstephenson/bats.git /tmp/bats
export PATH=$PATH:/tmp/bats/bin
cd $GOPATH/src/github.com/oleiade/trousseau/ && make test

echo "Your box is ready to use, enjoy!"
