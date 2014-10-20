#!/bin/bash


echo "=== installing system packages ==="
sudo apt-get update -qq -y
sudo apt-get install -qq -y git wget vim nano build-essential pkg-config mercurial subversion
echo "=== done ==="

echo "=== installing go lang tools ==="
wget --quiet https://storage.googleapis.com/golang/go1.3.3.linux-amd64.tar.gz
sudo tar -C /usr/local/ -xzf go1.3.3.linux-amd64.tar.gz

sudo echo "export GOROOT=/usr/local/go" >> /etc/profile
sudo mkdir -p /usr/local/gopath/{src,bin,pkg} && echo "export GOPATH=/usr/local/gopath" >> /etc/profile

sudo echo "export PATH=$PATH:$GOROOT/bin" >> /etc/profile
sudo echo "export PATH=$PATH:$GOPATH/bin" >> /etc/profile

rm -rf go1.3.3.linux-amd64.tar.gz
echo "=== done ==="


echo "=== installing the test gpg keys ==="
gpg --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_public_first_test.key
gpg --allow-secret-key-import --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_private_first_test.key
gpg --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_public_second_test.key
gpg --allow-secret-key-import --import /usr/local/gopath/src/github.com/oleiade/trousseau/tests/keys/trousseau_private_second_test.key
echo "=== done ==="

echo "=== building trousseau ==="
export GOPATH=/usr/local/gopath
cd /usr/local/gopath/src/github.com/oleiade/trousseau/ && make
echo "=== done ==="

echo "Your box is ready to use, enjoy!"
