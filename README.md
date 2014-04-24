[![Stories in Ready](https://badge.waffle.io/oleiade/trousseau.png?label=ready&title=Ready)](https://waffle.io/oleiade/trousseau)
![Trousseau, a portable encrypted keyring](https://dl.dropboxusercontent.com/u/2497327/Github/trousseau/trousseau-animation.gif)

## What

*Trousseau* is a **gpg** encrypted key-value store designed to be a *simple*, *safe* and *trustworthy* place for your data.
It stores data in a single multi-recipients encrypted file and can supports both local and remote storage sources (S3, ssh, gist so far) import/export.

Create a *trousseau* store, specify which *gpg* recipients are allowed to open and modify it, add some key-value pairs to it, export it to S3 for example, and re-import it on another device. As simple as that.

Whether you're a devops, a paranoid guy living in a bunker, or the random user who seeks a simple way to store it's critical data in secured manner. *Trousseau* can do something for you.

<div class="section-break"></div>
## Why

Storing, transporting, and sharing sensitive data can be hard, and much more difficult when it comes to automate it.

*Trousseau* was created with private keys transportation and sharing across a servers cluster in mind.
However it has proved being useful to anyone who need to store and eventually share a passwords store, bank accounts details or even more sensitive data.

<div class="subsection-break"></div>
### Real world use cases

<div class="subsection-break"></div>
#### For the devops out there

*Trousseau can be useful to you when it comes to*:

* **Store** sensitive data: Your brand new shiny infrastructure surely relies on many certificates and private keys of different kinds: ssl, rsa, gpg, ... *Trousseau* provides a simple and fine-tuned way to store their content in a single file that you can safely version using your favorite cvs. No more plain certificates and keys in your repositories and configuration files.
* **Share** passwords, keys and other critical data with co-workers and servers in your cluster in a safe manner. *Trousseau* encrypts its content for the specific recipient you provide it. Only the recipient you intend will be able to import and read-write the *Trousseau* store content. *Trousseau* proved itself to be a great way to share some services passwords with your co-workers too!
* **Deploy** keys to your servers in a safe and normative way. Encrypt the trousseau store for each server selectively.

<div class="subsection-break"></div>
#### For the common users

* **Store** your sensitive data like passwords, bank account details, sex tapes involving you and your teachers or whatever comes to your mind in an encrypted store.
* **Sync** your sensitive data store to remote services and easily share it between your unix-like devices.

## It's open-source

*Trousseau* is open source software under the MIT license.
Any hackers are welcome to supply ideas, features requests, patches, pull requests and so on.
Let's make *Trousseau* awesome!

See **Contribute** section.

<div class="section-break"></div>
## Installation

<div class="subsection-break"></div>
### Debian and ubuntu

A binary debian repository provides *trousseau* packages for *i386*, *x86_64* and *arm* architectures, so you can easily install it.
Just add the repository to your sources.list:

```bash
$ echo "deb http://dl.bintray.com/oleiade/deb /" | sudo tee /etc/apt/sources.list.d/trousseau.list
```

And you're ready to go:

```bash
$ sudo apt-get update && sudo apt-get install trousseau
```

<div class="subsection-break"></div>
### OSX

#### Homebrew

If you're using homebrew just proceed to installation using the provided formula:

```bash
$ brew install trousseau.rb
```

*Et voila!*

#### Macports

Coming soon (Don't be shy, if you feel like you could do it, just send pull request ;) )

<div class="subsection-break"></div>
### Build it

1. First, make sure you have a [Go](http://http://golang.org/) language compiler **>= 1.2** (*mandatory*) and [git](http://gitscm.org) installed.
2. Make sure you have the following go system dependencies in your ``$PATH``: ``bzr, svn, hg, git``
3. Ensure your [GOPATH](http://golang.org/doc/code.html#GOPATH) is properly set.
4. Run ``make``

<div class="section-break"></div>
## Prerequisities

<div class="subsection-break"></div>
### Gpg passphrase

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

##### Environment variable

Alternatively, you can pass your primary key passphrase as `TROUSSEAU_PASSPHRASE` environment variable:

```bash
$ export TROUSSEAU_PASSPHRASE=mysupperdupperpassphrase
$ trousseau get abc
```

##### Passphrase global option

Ultimately, you can pass you gpg passphrase through the command line global option:

```bash
$ trousseau --passhphrase mysupperdupperpassphrase get abc
```

### Environment

Trousseau behavior can be controlled through the system environment:

* *TROUSSEAU_STORE* : if you want to have multiple trousseau data store, set this environment variable to the path of the one you want to use. Default is ``$HOME/.trousseau``

<div class="section-break"></div>
## Let's get started

<div class="subsection-break"></div>
### Basics

First use of **trousseau** requires the data store to be created. A **trousseau** data store is built and maintained for a list of *gpg* recipients who will be the only ones able to decrypt and manipulate it (so don't forget to include yourself ;) )

<div class="break"></div>
#### API

* **create** [RECIPIENTS ...] : creates the trousseau encrypted datastore for provided recipients and stores it in `$HOME/.trousseau`
* **meta** : Outputs the store metadata.
* **add-recipient** RECIPIENT : Adds a recipient to the store. The recipient will be able to open and modify the store.
* **remove-recipient** RECIPIENT : Removes a recipient from the store. The recipient will not be able to open or modify the store.

<div class="break"></div>
#### First steps with the data store

```bash
# create a trousseau for two gpg recipients
# both key ids and key email are supported.
$ trousseau create 4B7D890,foo@bar.com 
trousseau created at $HOME/.trousseau
```

Trousseau data store consists in single gpg encrypted file residing in your ``$HOME`` directory. Check by yourself.

```bash
$ cat ~/.trousseau
-----BEGIN PGP MESSAGE-----
wcBMA5i2a4x3jHQgAQgAGKAZd5UFauGBMkFz7wi4v4aNTGGpDS81drrevo/Tntdz
rr+PR/GjUlKZxhvG18mr+FuTV6q2DOK3Z0nROs57PLK9Q3ye40Su/Af1vj+LaN4i
AAMK9YVpjKaxz+pciUm8nBDkRxp3CLZ9eA2B+1JBy5HgziHY+7KC/dvaubRv0M0J
qzYvshIYU0urVQt7oO4WYVQbJ1N0OXV3oAzW4bBBs/p6b8KSUlmvHUr+9r4V1KvU
ynpHbp1T2HVPC9uqLgJ+PRjlQ2QsxjezkBntOFMaeMZjq2m2glw90aIGDAPjkMKy
42qQbmdrT3+houqeKUrLcVFNOxevVEZLf8N3Qgo/H9LgAeSroddqYkJzOmknxDzP
MDk+4TaY4Ljge+G7j+CB4iBsIjrgSefl/4ZU30dJ/DHyL5i3lCCGXXAo2eqfJg2w
FZgh+qc8Mbjlz2iMdnC+b8rRwhMTgD1Tyd8vbR1ArPfQh3ThdePwrdyE86CYQZOA
MIBfKgTUpWiAtEhM23melF8H3oznrIKt1ZtDsxJEuBCZ86XlC9TF27XFWbnl7rfK
jF2kqP3DuuBA5d23HprbN6LjDSJeKbXDvc5LetBI7O5y954n3tMWCB9y4EjkpVAx
EWnovjEnnW89uXHaFOBQ4naH4kjg1OHEquCf4Nvgl+S5Pfi875yAKqxxK/+e8GGo
4q8UZC7ho/cA
=t2zr
-----END PGP MESSAGE-----
```

<div class="break"></div>
### Manipulating keys

Once your trousseau has been created, you're now able to read, write, list, delete its data. Here's how the fun part goes.

<div class="break"></div>
#### API

* **get** KEY [--file]: Outputs the stored KEY-value pair, whether on *stdout* or in pointed ``--file`` option path.
* **set** KEY [VALUE | --file] : Sets the provided key-value pair in store using provided value or extracting it from path pointed by ``--file`` option.
* **del** KEY : Deletes provided key from the store
* **keys** : Lists the stored keys
* **show** : Lists the stored key-value pairs

<div class="break"></div>
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

$ trousseau show
abc: 123
easy as: do re mi
myuser.ssh.public_key: ssh-rsa 1289eu102ij30192u3e0912e
...

# Whenever you want to export a key value to a file, just use
# the get command --file option
$ trousseau get myuser.ssh.public_key --file /home/myuser/id_rsa.pub

# Now if you don't need a key anymore, just drop it.
$ trousseau del abc  # Now the song lacks something doesn't it?
```

<div class="break"></div>
### Importing/Exporting to remote storage

Trousseau was built with data remote storage in mind. Therefore it provides *push* and *pull* actions to export and import the trousseau data store to remote destinations.
As of today S3, SSH and gist storages are available (more are to come).

<div class="break"></div>
#### API

* **push** : Pushes the trousseau data store to remote storage
* **pull** : Pulls the trousseau data store from remote storage

<div class="break"></div>
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

<div class="break"></div>
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

<div class="break"></div>
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

<div class="subsection-break"></div>
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

<div class="break"></div>
### Metadata

Trousseau keeps track and exposes all sort of metadata about your store that you can access through the ``meta`` command.

```bash
$ trousseau meta
CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
Recipients: [4B7D890,28EA78B]
TrousseauVersion: 0.1.0c
```

Once again, if you're intersted in how the meta data are stored, go check yourself by decrypting the store content using one of your recipients private key.

```bash
$ cat ~/.trousseau | gpg -d -r 4B7D890 --textmode
You need a passphrase to unlock the secret key for
user: "My Gpg User <MyGpg@mail.com>"
2048-bit RSA key, ID 4B7D890, created 2013-05-21 (main key ID 4B7D890)

gpg: encrypted with 2048-bit RSA key, ID 4B7D890, created 2013-05-21
  "My Gpg User <MyGpg@mail.com>"
{"_meta":{"created_at":"2013-08-12 08:00:20.457477714 +0200 CEST","last_modified_at":"2013-08-12 08:00:20.457586991 +0200 CEST","recipients":["92EDE36B"],"version":"0.1.0"},"data":{}}
```

<div class="break"></div>
### Adding and removing recipients

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

<div class="section-break"></div>
## More features to come

* Support for Sftp remote storage
* Support for GDrive remote storage
* Support for Dropbox remote storage

* In a further future I might add TrueCrypt encryption

<div class="section-break"></div>
## Contribute

* Check for open issues or open a fresh issue to start a discussion around a feature idea or a bug.
* Fork the repository on GitHub to start making your changes to the **master** branch (or branch off of it).
* Write tests which show that the bug was fixed or that the feature works as expected.
* Send a pull request and bug the maintainer until it gets merged and published. :) Make sure to add yourself to AUTHORS.

## Changelog

See [History](https://github.com/oleiade/trousseau/blob/master/History.md)



[![Bitdeli Badge](https://d2weczhvl823v0.cloudfront.net/oleiade/trousseau/trend.png)](https://bitdeli.com/free "Bitdeli Badge")

