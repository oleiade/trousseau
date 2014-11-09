#!/bin/bash

GOROOT=/usr/local/go
GOPATH=/usr/local/gopath

echo "---> installing system packages"
sudo apt-get update -qq -y > /dev/null
sudo apt-get install -qq -y git wget vim nano build-essential pkg-config mercurial subversion > /dev/null
echo "<--- done"

if [ ! -d $GOROOT ]; then
	echo "---> installing go lang tools"
	wget --quiet https://storage.googleapis.com/golang/go1.3.3.linux-amd64.tar.gz > /dev/null
	sudo tar -C /usr/local/ -xzf go1.3.3.linux-amd64.tar.gz > /dev/null

	sudo echo "export GOROOT=$GOROOT" >> /etc/profile
	sudo mkdir -p /usr/local/gopath/{src,bin,pkg} && echo "export GOPATH=$GOPATH" >> /etc/profile

	sudo echo "export PATH=$PATH:$GOROOT/bin" >> /etc/profile 
	sudo echo "export PATH=$PATH:$GOPATH/bin" >> /etc/profile

	rm -rf go1.3.3.linux-amd64.tar.gz > /dev/null
	echo "<--- done"
fi


echo "---> installing the test gpg keys"
gpg --quiet --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_public_first_test.key &> /dev/null
gpg --quiet --allow-secret-key-import --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_private_first_test.key &> /dev/null
gpg --quiet --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_public_second_test.key &> /dev/null
gpg --quiet --allow-secret-key-import --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_private_second_test.key &> /dev/null
echo "<--- done"

echo "---> building trousseau"
export GOPATH=$GOPATH
export PATH=$PATH:$GOROOT/bin
cd /usr/local/gopath/src/github.com/oleiade/trousseau/ && make
echo "<--- done"

echo "Your box is ready to use, enjoy!"
