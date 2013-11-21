![Trousseau, a portable encrypted keyring](https://dl.dropboxusercontent.com/u/2497327/Github/trousseau/trousseau-animation.gif)

## What

*Trousseau* is a **gpg** encrypted key-value store designed to be a *simple*, *safe* and *trustworthy* place for your data.
It stores data in a single multi-recipients encrypted file and can supports both local and remote storage sources (S3 and ssh so far) import/export.

Create a *trousseau* store, specify which *gpg* recipients are allowed to open and modify it, and adding some key-value pairs to, export it to s3 for example, and re-import it on another device. As simple as that.

Whether you're a devops, a paranoid guy living in a bunker, or the random user who seek a simple way to store it's critical data in secured manner. *Trousseau* can do something for you.

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

* **Store** sensitive data: Your brand new shinny infrastructure surely relies on many certificates and private keys of different kinds: ssl, rsa, gpg, ... *Trousseau* provides a simple and fine-tuned way to store their content in a single file that you can safely version using your favorite cvs. No more plain certificates and keys in your repositories and configuration files.
* **Share** passwords, keys and other critical data with coworkers and servers in your cluster in a safe maneer. *Trousseau* encrypts its content for the specific recipient you provide it. Only the recipient you intend will be able to import and read-write the *Trousseau* store content. *Trousseau* proved itself to be a great way to share some services passwords with your coworkers too!
* **Deploy** keys to your servers in a safe and normative way. Encrypt the trousseau store for every servers selectively.

<div class="subsection-break"></div>
#### For the common users

* **Store** your sensitive data like passwords, bank account details, sex tapes involving you and your teachers or whatever comes to your mind in an encrypted store.
* **Sync** your sensitive data store to remote services and easily share it between your unix-like devices.

## It's open-source

*Trousseau* is an open source software under the MIT license.
Any hackers are welcome to supply ideas, features requests, patches, pull requests and so on: see **Contribute**

<div class="section-break"></div>
## Installation

<div class="subsection-break"></div>
### Debian and ubuntu

A bintray debian repository provides *trousseau* packages for *i386*, *x86_64* and *arm* architectures, so you can easily install it.
Just add the repository to your sources.list:

```bash
$ sudo echo "deb http://dl.bintray.com/oleiade/deb /" >> /etc/apt/sources.list
```

And you're ready to go:

```bash
$ sudo apt-get install trousseau
```

<div class="subsection-break"></div>
### OSX

A repository for osx distributions will be provided soon. But for now, please refer to the **build** installation.

<div class="subsection-break"></div>
### Build it

1. First, make sure you have a `Go <http://http://golang.org/>` language compiler **>= 1.1.2** (*mandatory*) and `git <http://gitscm.org>` installed.
2. Make sure you have the following go system dependencies in your `$PATH`: `bzr, svn, hg, git`
3. Then, just build and copy the `./bin/trousseau` executable to a system *PATH* location

```bash
make
sudo cp ./bin/trousseau /usr/local/bin/trousseau
```

<div class="section-break"></div>
## Prerequisities

<div class="subsection-break"></div>
### Gpg password

Every decryption operations will require your *gpg* primary key password. As of today, **trousseau** will handle your password through the environment.
Export your primary key password as `TROUSSEAU_PASSWORD` environment variable.

*Example*:

```bash
$ export TROUSSEAU_PASSWORD=mysupperdupperpassword
$ trousseau get abc
```

<div class="subsection-break"></div>
### AWS credentials

If you intend to use the push/pull feature using `S3 <http://http://aws.amazon.com/s3/>` service, please make sure to set the
`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` variables, like:

```bash
$ export AWS_ACCESS_KEY_ID=myaeccskey && export AWS_SECRET_ACCESS_KEY=mysecretkey
$ trousseau pull
```

<div class="subsection-break"></div>
### Environment variables (so you know)

* `TROUSSEAU_PASSWORD` (**mandatory**): your *gpg* primary key password that will be used to identify you as one of the trousseau data store recipient and give read/write access.
* `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` (*optional*): Your aws account credentials with proper read/write acces over S3. *Only if you intend to use the S3 remote storage features*
* `TROUSSEAU_S3_BUCKET` and `TROUSSEAU_S3_FILENAME` (*optional*): The remote s3 bucket the trousseau data should be pushed/pulled from and the expected remote name of the trousseau data store file.

<div class="section-break"></div>
## Let's get started

<div class="subsection-break"></div>
### Basics

First use of **trousseau** requires the data store to be created. A **trousseau** data store is built and maintained for a list of *gpg* recipients who will be the only ones able to decrypt and manipulate it (so don't forget to include yourself ;) )

<div class="break"></div>
#### Api

* **create** [RECIPIENTS ...] : creates the trousseau encrypted datastore for provided recipients and stores it in `$HOME/.trousseau`
* **meta** : Outputs the store metadata.
* **add-recipient** RECIPIENT : Adds a recipient to the store. The recipient will be able to open and modify the store.
* **remove-recipient** RECIPIENT : Removes a recipient from the store. The recipient will not be able to open or modify the store.

<div class="break"></div>
#### First steps with the data store

```bash
$ trousseau create 4B7D890,28EA78B  # create a trousseau for two gpg recipients
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

Once your trousseau has been created, you're now able to read, write, list, delete it's data. Here's how the fun part goes.

<div class="break"></div>
#### Api

* **get** KEY : Outputs the stored KEY-value pair
* **set** KEY VALUE : Sets the provided key-value pair in store
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
$ trousseau set oleiade-private-key "`cat ~/.ssh/id_rsa`"


# Now let's make sure data has been added
$ trousseau keys
abc
easy as
oleiade-private-key

$ trousseau get abc
123

$ trousseau show
abc: 123
easy as: do re mi
oleiade-private-key: --- BEGIN PRIVATE KEY ---
...


# Now if you don't need a key anymore, just drop it.
$ trousseau del abc  # Now the song lacks something doesn't it?
```

<div class="break"></div>
### Importing/Exporting to remote storage

Trousseau was built with data remote storages in mind. As of today only S3 and SSH storages are available, but more are to come (don't forget to set your aws credentials environment variables)

<div class="break"></div>
#### Api

* **push** : Pushes the trousseau data store to remote storage
* **pull** : Pulls the trousseau data store from remote storage

<div class="break"></div>
#### S3 Example

Pushing the trousseau data store to Amazon S3 will require some setup:

* First, Make sure you've set up the aws credentials environment variables like described in the configuration section of this README.
* Then you can setup the bucket to push data store into and the remote filename using environment. However, you're also able to provide these parameters as arguments of the **push** and **pull** methods.

```bash
$ export TROUSSEAU_S3_FILENAME=trousseau
$ export TROUSSEAU_S3_BUCKET=mytrousseaubucket
```

Now that everything is configured properly, you're ready to properly push the data store to S3.

```bash
# Considering a non empty trousseau data store
$ trousseau show
abc: 123
easy as: do re mi

# And then you're ready to push
$ trousseau push


# Now that data store is pushed to S3, let's remove the
# local data store and pull it once again to ensure it worked
$ rm ~/.trousseau
$ trousseau show
Trousseau unconfigured: no data store

$ trousseau pull
$ trousseau show
abc: 123
easy as: do re mi
```

<div class="break"></div>
#### Scp example

*Trousseau* allows you to push your data store to a ssh location. It doesn't need any special setup. So here we can go with a complete example.

```bash
# We start with a non empty trousseau data store
$ trousseau show
abc: 123
easy as: do re mi

# To push it using scp we need to provide it a couple of
# basic options
$ trousseau push --remote-storage scp --host <myhost> --port <myport> --user <myuser>


# Now that data store has been pushed to the remote storage
# using scp, let's remove the local data store and pull it
# once again to ensure it worked
$ rm ~/.trousseau
$ trousseau show
Trousseau unconfigured: no data store

$ trousseau pull --remote-storage scp --host <myhost> --port <myport> --user <myuser>
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

* In a further future I might add truecrypt encryption

<div class="section-break"></div>
## Contribute

* Check for open issues or open a fresh issue to start a discussion around a feature idea or a bug.
* Fork the repository on GitHub to start making your changes to the **master** branch (or branch off of it).
* Write tests which shows that the bug was fixed or that the feature works as expected.
* Send a pull request and bug the maintainer until it gets merged and published. :) Make sure to add yourself to AUTHORS.



[![Bitdeli Badge](https://d2weczhvl823v0.cloudfront.net/oleiade/trousseau/trend.png)](https://bitdeli.com/free "Bitdeli Badge")

