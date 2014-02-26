# -*- mode: ruby -*-
# vi: set ft=ruby :

# Vagrantfile API/syntax version. Don't touch unless you know what you're doing!
VAGRANTFILE_API_VERSION = "2"

Vagrant.configure(VAGRANTFILE_API_VERSION) do |config|
  config.vm.box = "precise64"
  config.vm.box_url = "http://files.vagrantup.com/precise64.box"
  config.vm.synced_folder ".", "/home/vagrant"
  config.vm.provision :shell, :path => "scripts/vagrant_provision.sh"

  config.vm.provider :virtualbox do |vb|
    vb.name = "trousseau_vm"
    vb.gui = false
    vb.customize ["modifyvm", :id, "--memory", "128"]
  end
end
