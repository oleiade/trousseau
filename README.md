![Trousseau, a portable encrypted keyring](trousseau.gif)

[![Build Status](https://travis-ci.org/oleiade/trousseau.svg?branch=master)](https://travis-ci.org/oleiade/trousseau)

## What

*Trousseau* is an encrypted key-value store designed to be a *simple*, *safe* and *trustworthy* place for your data.

It stores data in a **single encrypted file**.
It supports both **asymetric encryption** using OpenPGP, and **symmetric encryption** using AES256.
It can be easily synced across devices using Dropbox, OneDrive...
It can be exported and imported to/from multiple remote storages using integrated S3, ssh, and gist support.
If used with OpenPGP encryption, it is able to restrict access to the data store to a set of recipients.

Create a *trousseau* data store, add some key-value pairs to it, push it to S3 and re-import it from another device or simply sync it over Dropbox. Safe data sharing had never been that simple!

*Secrets are made to be shared, just not with anyone.* Whether you're an admin, a paranoid guy living in a bunker, or a random user who seeks a simple way to store it's critical data in secured manner. *Trousseau* can do something for you.

## Why

Storing, transporting, and sharing sensitive data can be hard, and much more difficult when it comes to automate it.

*Trousseau* was created with private keys and certificates (such as private keys) sharing across a cluster in mind.
However it has proved being useful to anyone who need to store and eventually share a passwords store, bank accounts details or even more sensitive data.

### Use cases

#### For admins and ops

*Trousseau can be useful to you when it comes to*:

* **Store** sensitive data: No more plain certificates and keys in your repositories and configuration files. Your brand new shiny infrastructure surely relies on many certificates and private keys of different kinds: ssl, rsa, gpg, ... *Trousseau* provides a simple and fine-tuned way to store their content in a single file that you can safely version using your favorite version control system.
* **Share** passwords, keys and other critical data with co-workers and servers in your cluster in a safe manner. *Trousseau* can encrypt its content for specific recipients you provide to it (Only the recipient you intend will be able to import and read-write the *Trousseau* store content). *Trousseau* proved itself to be a great way to share some services passwords with your co-workers too! Simply set up a trousseau store with symmetric encryption, sync it over dropbox, et voila!
* **Deploy** keys to your servers in a safe and normative way. Encrypt the trousseau store for each server selectively.

#### For the common users

* **Store** your sensitive data such as passwords, bank account details or bitcoin wallets in an encrypted store.
* **Sync** your sensitive data store to remote services and easily share it between your devices.

## How

### Installation

#### Debian and ubuntu

A binary debian repository provides *trousseau* packages for *i386*, *x86_64* and *arm* architectures, so you can easily install it.
Just add the repository to your sources.list:

```bash
$ echo "deb https://dl.bintray.com/oleiade/deb /" | sudo tee /etc/apt/sources.list.d/trousseau.list
```

And you're ready to go:

```bash
$ sudo apt-get update && sudo apt-get install trousseau
```

#### OSX

##### Homebrew

If you're using homebrew just proceed to installation using the provided formula:

```bash
$ brew install https://raw.githubusercontent.com/oleiade/trousseau/master/trousseau.rb
```

##### Binaries

Get the latest darwin release zip archive from the repository. Unzip it, and place the trousseau executable wherever it suits you.

```bash
$ unizp trousseau_X.Y.Z_darwin_amd64.zip
$ cp trousseau_X.Y.Z_darwin_amd64/trousseau /usr/local/binary
```

#### Build it

1. First, make sure you have a [Go](http://http://golang.org/) language compiler **>= 1.5** (*mandatory*) and [git](http://gitscm.org) installed.
2. Make sure you have the following go system dependencies in your ``$PATH``: ``bzr, svn, hg, git``
3. Ensure your [GOPATH](http://golang.org/doc/code.html#GOPATH) is properly set.
4. Run ``make``
5. The trousseau binary is now in the `bin/` folder

### Prerequisites

#### If you go for OpenPGP asymmetric encryption

Every decryption operations will require your *gpg* primary key passphrase.
As of today, **trousseau** is able to handle your passphrase through multiple ways:
* system's keyring manager
* gpg-agent daemon
* system environment
* ``--passphrase`` global option

##### Keyring manager

Supported system keyring manager are osx keychain access and linux gnome secret-service and gnome-keychain (more might be added in the future on demand).
To use the keyring manager you will need to set up the ``TROUSSEAU_KEYRING_SERVICE`` environment variable to the name of they keyring manager key holding the trousseau main gpg key passphrase.

```bash
$ export TROUSSEAU_KEYRING_SERVICE=my_keyring_key
$ trousseau get abc
```

##### Gpg agent

Another authentication method supported is gpg-agent. In order to use it make sure you've started the gpg-agent daemon and exported the ``GPG_AGENT_INFO`` variable, trousseau will do the rest.

```bash
$ export GPG_AGENT_INFO=path_to_the_gpg_agent_info_file
$ export TROUSSEAU_MASTER_GPG_ID=myid@mymail.com
$ trousseau get abc
```

#### Whatever encryption style you go for

##### Environment variable

You can pass your primary key passphrase as `TROUSSEAU_PASSPHRASE` environment variable:

```bash
$ export TROUSSEAU_PASSPHRASE=mysupperdupperpassphrase
$ trousseau get abc
```

##### ask-passphrase global option

You can have trousseau asking for your passphrase using the command line global option:

```bash
$ trousseau --ask-passhphrase get abc
Passphrase:
123
```

#### Environment

Trousseau behavior can be controlled through the system environment:

* *TROUSSEAU_STORE* : if you want to have multiple trousseau data store, set this environment variable to the path of the one you want to use. Default is ``$HOME/.trousseau``

## Let's get started

### Basics

#### API

#### First steps with the data store

First step with **trousseau** is to create a data store.

To do so, you will need to decide the kind of encryption you wish to use:
+ OpenPGP asymmetric encryption: accessing the data store will be restricted to the recipients (gpg) its been encrypted for. This is probably the best choice if you intend to share the data store with multiple servers or gpg capable devices. It can also be a good choice if you inted to share the data store with a team or selected people.
+ AES256 symmetric encryption: the data store will be encrypted using a passphrase you will provide. This is probably the best choice if you intend to store sensitive personal informations (passwords, bank details, bitcoins...) and sync it accross devices.  

Then, you can proceed and create a data store with the `create` command.
As a default:
+ data stores will be created as `$HOME/.trousseau`. However the global option `store` will allow you to select the place on the filesystem where *trousseau* should create/open the data store.
+ data stores will be created using asymmetric OpenPGP encryption. However `encryption-type` and  `encryption-algorithm` options will allow to select explicitly the encryption mode of your choice.

##### HOWTO

```bash
# create a trousseau for two gpg recipients
# both key ids and key email are supported.
$ trousseau create 4B7D890,foo@bar.com
trousseau created at $HOME/.trousseau

# Or create a symmetrically encrypted data store
# with a passphrase
$ trousseau create --encryption-type symmetric
Passphrase:
trousseau created at $HOME/.trousseau
```

Trousseau data store consists in a single encrypted file residing in your ``$HOME`` directory. Check by yourself.

```bash
$ cat ~/.trousseau
{
    "crypto_type":1,
    "crypto_algorithm":0,
    "_data":"012ue091ido19d81j2d01029dj1029d1029u401294i ... 1028019k0912djm0129d12"
}
```

If you've just updated trousseau to a version marked as implying backward incompatibilities, the ``upgrade`` command is here to help

```bash
$ trousseau upgrade
Upgrading trousseau data store to version M: success
Upgrading trousseau data store to version N: success
# This is it, your legacy data store has now been upgraded to be compatible with
# your current version of trousseau
```

### Manipulating keys

Once your trousseau has been created, you're now able to read, write, list, delete its data. Here's how the fun part goes.

#### You've got the keys

```bash
# Right now the store is empty
$ trousseau show


# Let's add some data into it
$ trousseau set abc 123
$ trousseau set "easy as" "do re mi"

# set action supports a --file flag to use the content
# of a file as value
$ trousseau set myuser.ssh.public_key --file ~/.ssh/id_rsa.pub


# Now let's make sure data has been added
$ trousseau keys
abc
easy as
myuser.ssh.public_key

# Let's check values too
$ trousseau get abc
123

# What about renaming abc key, just for fun?
$ trousseau rename abc 'my friend jackson'
$ trousseau keys
my friend jackson
easy as
myuser.ssh.public_key


$ trousseau show
my friend jackson: 123
easy as: do re mi
myuser.ssh.public_key: ssh-rsa 1289eu102ij30192u3e0912e
...

# Whenever you want to export a key value to a file, just use
# the get command --file option
$ trousseau get myuser.ssh.public_key --file /home/myuser/id_rsa.pub

# Now if you don't need a key anymore, just drop it.
$ trousseau del 'my friend jackson' # Now the song lacks something doesn't it?
```

#### API

* **get** KEY [--file]: Outputs the stored KEY-value pair, whether on *stdout* or in pointed ``--file`` option path.
* **set** KEY [VALUE | --file] : Sets the provided key-value pair in store using provided value or extracting it from path pointed by ``--file`` option.
* **rename** KEY_NAME NEW_NAME : Renames a store key
* **del** KEY : Deletes provided key from the store
* **keys** : Lists the stored keys
* **show** : Lists the stored key-value pairs

### Importing/Exporting to remote storage

Trousseau was built with data remote storage in mind. Therefore it provides *push* and *pull* actions to export and import the trousseau data store to remote destinations.
As of today S3, SSH and gist storages are available (more are to come).

#### API

* **push** : Pushes the trousseau data store to remote storage
* **pull** : Pulls the trousseau data store from remote storage

#### DSN

In order to make your life easier trousseau allows you to select your export and import sources using a *DSN*.

```
    {protocol}://{identifier}:{secret}@{host}:{port}/{path}
```

* **protocol**: The remote service target type. Can be one of: *s3* or *scp*
* **identifier**: The login/key/whatever to authenticate **trousseau** to the remote service. Provide your *aws_access_key* if you're targeting *s3*, or your remote login if you're targeting *scp*.
* **secret**: The secret to authenticate **trousseau** to the remote service. Provide your *aws_secret_key* if you're targeting *s3*, or your remote password if you're targeting *scp*.
* **host**: Your bucket name is you're targeting *s3*. The host to login to using *scp* otherwise.
* **port**: The *aws_region* if you're targeting *s3*. The port to login to using *scp* otherwise.
* **path**: The remote path to push to or retrieve from the trousseau file on a ``push`` or ``pull`` action.

#### S3 Example

```bash
# Considering a non empty trousseau data store
$ trousseau show
abc: 123
easy as: do re mi

# And then you're ready to push
$ trousseau push s3://aws_access_key:aws_secret_key@bucket:region/remote_file_path


# Now that data store is pushed to S3, let's remove the
# local data store and pull it once again to ensure it worked
$ rm ~/.trousseau
$ trousseau show
Trousseau unconfigured: no data store

$ trousseau pull s3://aws_access_key:aws_secret_key@bucket:region/remote_file_path
$ trousseau show
abc: 123
easy as: do re mi
```

#### Scp example

```bash
# We start with a non-empty trousseau data store
$ trousseau show
abc: 123
easy as: do re mi

# To push it using scp we need to provide it a couple of
# basic options.
# Nota: In order for your remote password not to appear
# in your shell history, we strongly advise you to use
# the push/pull --ask-password option instead of supplying
# the password through the dsn.
$ trousseau push --ask-password scp://user:@host:port/remote_file_path
Password:
Trousseau data store succesfully pushed to ssh remote storage


# Now that data store has been pushed to the remote storage
# using scp, let's remove the local data store and pull it
# once again to ensure it worked
$ rm ~/.trousseau
$ trousseau show
Trousseau unconfigured: no data store

$ trousseau pull --ask-password scp://user:@host:port/remote_file_path
Password:
Trousseau data store succesfully pulled from ssh remote storage

$ trousseau show
abc: 123
easy as: do re mi
```

#### Gist example

To use the gist remote storage support, you will need to generate a Github [personal access token](https://github.com/settings/applications#personal-access-tokens).
Once you've generated one, use it as the dsn *password* field as in the following example:

```bash
# We start with a non-empty trousseau data store
$ trousseau show
abc: 123
easy as: do re mi

# Nota:
# * Gist remote storage doesn't use the host and port dsn fields,
#   but you still need to provide their ':' separator
$ trousseau push gist://user:mysuppedupertoken@:/gist_name
Password:
Trousseau data store succesfully pushed to gist remote storage


# Now that data store has been pushed to gist
# let's remove the local data store and pull it
# once again to ensure it worked
$ rm ~/.trousseau
$ trousseau show
Trousseau unconfigured: no data store

$ trousseau pull gist://user:mysupperduppertoken@:/gist_name
Password:
Trousseau data store succesfully pulled from gist

$ trousseau show
abc: 123
easy as: do re mi
```

### Local imports and exports

#### API

* **import** FILENAME: will import a trousseau data store from the local filesystem. The operation **erases** the current trousseau store content.
* **export** FILENAME: will export the current trousseau data store as `FILENAME` on the local fs.

#### Real world example

```bash
$ trousseau export testtrousseau.asc  # Fine we've exported our current data store into a single file
$ mail -f testtrousseau.asc cousin@machin.com  # Let's pretend we've sent it by mail

# Now cousin machin is now able to import the data store
$ trousseau import testtrousseau.asc
$ trousseau show
cousin_machin:isagreatbuddy
adams_family:rests in peace, for sure
```

### Metadata

Trousseau keeps track and exposes all sort of metadata about your store that you can access through the ``meta`` command.

```bash
$ trousseau meta
CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
Recipients: [4B7D890,28EA78B]
TrousseauVersion: 0.1.0c
```

### (OpenPGP encryption) Adding and removing recipients

Okay, so you've created a trousseau data store with two recipients allowed to manipulate it. Now suppose you'd like to add another recipient to be able to open and update the trousseau store; or to remove one.
``add-recipient`` and ``remove-recipient`` commands can help you with that.

```bash
$ trousseau add-recipient 75FE3AB
$ trousseau add-recipient 869FA4A
$ trousseau meta
CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
Recipients: [4B7D890, 75FE3AB, 869FA4A]
TrousseauVersion: 0.1.0c

$ trousseau remove-recipient 75FE3AB
$ trousseau meta
CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
Recipients: [4B7D890, 869FA4A]
TrousseauVersion: 0.1.0c
```

## Contribute

For detailed contribution instructions, see the the [CONTRIBUTING](https://github.com/oleiade/trousseau/blob/master/CONTRIBUTING.md) document

However here is a quick summary for all of you in a hurry:
* Check for open issues or open a fresh issue to start a discussion around a feature idea or a bug.
* Fork the repository on GitHub.
* Start a branch from **develop** dedicated to your changes.
* If relevant: write tests showing the bug was fixed or the feature implemented works as expected.
* Send a pull request and bug the maintainer until it gets merged and published. :) Make sure to add yourself to AUTHORS.

## It's open-source

*Trousseau* is open source software under the MIT license.
Any hackers are welcome to supply ideas, features requests, patches, pull requests and so on.
Let's make *Trousseau* awesome!

See **Contribute** section.

## Changelog

See [CHANGELOG](https://github.com/oleiade/trousseau/blob/master/CHANGELOG.md)
