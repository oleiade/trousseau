#!/bin/bash


echo "=== installing system packages ==="
sudo apt-get update -qq -y
sudo apt-get install -qq -y git wget vim nano build-essential pkg-config mercurial subversion
echo "=== done ===\n"

echo "=== installing go lang tools ==="
wget --quiet http://go.googlecode.com/files/go1.1.1.linux-amd64.tar.gz
sudo tar -C /usr/local/ -xzf go1.1.1.linux-amd64.tar.gz
sudo echo "export PATH=$PATH:/usr/local/go/bin" >> /etc/profile
rm -rf go1.1.1.linux-amd64.tar.gz
echo "=== done ===\n"

echo "Your box is ready to use, enjoy!"
