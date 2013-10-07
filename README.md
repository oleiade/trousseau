## What it is


*Trousseau* is a **gpg** encrypted key-value store file written in Go. It is designed to be easily manipulated and imported/exported on multiple remote storage sources.
It was built with private keys transportation and sharing across servers in mind. However it could be useful to anyone who needs to store and eventualy share sensitive datas: passwords, banking credentials sensitive personal informations, and so on...
As of today *Trousseau* exposes a **push**-**pull** interface to *S3* and *scp* storage methods but more are to come (Ftp, Dropbox, GDrive).
*Trousseau* is an open source software under the MIT license. Any hackers are welcome to supply ideas, features requests, patches, pull requests and so on: see **Contribute**

<div class="section-break"></div>
## Installation

<div class="subsection-break"></div>
### Binaries

Precompiled binaries of the project for *i386*, *x86_64* and *arm* architectures (linux and darwin) can be found in the project *bin* folder.
Just copy it on your `PATH` and go ahead with *usage* instructions.

<div class="subsection-break"></div>
### Build it

1. First, make sure you have a `Go <http://http://golang.org/>` language compiler **>= 1.1.2** (*mandatory*) and `git <http://gitscm.org>` installed.
2. Then, just build and copy the `./bin/trousseau` executable to a system *PATH* location

```bash
    make
    sudo cp ./bin/trousseau /usr/local/bin/trousseau
```

<div class="section-break"></div>
## Usage

<div class="subsection-break"></div>
### Prerequisities

<div class="break"></div>
#### Gpg password

Every decryption operations will require your *gpg* primary key password. As of today, **trousseau** will handle your password through the environment.
Export your primary key password as `TROUSSEAU_PASSWORD` environment variable.

*Example*:

```bash
    $ export TROUSSEAU_PASSWORD=mysupperdupperpassword
    $ trousseau get abc
```

<div class="break"></div>
#### AWS credentials

If you intend to use the push/pull feature using `S3 <http://http://aws.amazon.com/s3/>` service, please make sure to set the
`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` variables, like:

```bash
    $ export AWS_ACCESS_KEY_ID=myaeccskey && export AWS_SECRET_ACCESS_KEY=mysecretkey
    $ trousseau pull
```

<div class="break"></div>
#### Environment variables

* `TROUSSEAU_PASSWORD` (**mandatory**): your *gpg* primary key password that will be used to identify you as one of the trousseau data store recipient and give read/write access.
* `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` (*optional*): Your aws account credentials with proper read/write acces over S3. *Only if you intend to use the S3 remote storage features*
* `TROUSSEAU_S3_BUCKET` and `TROUSSEAU_S3_FILENAME` (*optional*): The remote s3 bucket the trousseau data should be pushed/pulled from and the expected remote name of the trousseau data store file.

<div class="subsection-break"></div>
### Actions

<div class="break"></div>
#### Store creation and management

First use of **trousseau** requires the data store to be created. A **trousseau** data store is built and maintained for a list of *gpg* recipients who will be the only ones able to decrypt and manipulate it (so don't forget to include yourself ;) )

<div class="break"></div>
*Api*

* `create [RECIPIENTS ...]` : creates the trousseau encrypted datastore for provided recipients and stores it in `$HOME/.trousseau`
* `meta` : Outputs the store metadata.
* `add-recipient RECIPIENT` : Adds a recipient to the store. The recipient will be able to open and modify the store.
* `remove-recipient RECIPIENT` : Removes a recipient from the store. The recipient will not be able to open or modify the store.

<div class="break"></div>
*Create the trousseau datastore*

```bash
    # create a trousseau for two gpg recipients
    $ trousseau create 4B7D890,28EA78B
    trousseau created at $HOME/.trousseau


    # as you can see, trousseau data store consists
    # in only one encrypted file, in your $HOME
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
*Metadata*

```bash
    # If you take a look at the encrypted content of the
    # trousseau datastore manually using gpg, you can see
    # that the created trousseau is not empty 
    $ cat ~/.trousseau | gpg -d -r 4B7D890 --textmode
    You need a passphrase to unlock the secret key for
    user: "My Gpg User <MyGpg@mail.com>"
    2048-bit RSA key, ID 4B7D890, created 2013-05-21 (main key ID 4B7D890)

    gpg: encrypted with 2048-bit RSA key, ID 4B7D890, created 2013-05-21
      "My Gpg User <MyGpg@mail.com>"
    {"_meta":{"created_at":"2013-08-12 08:00:20.457477714 +0200 CEST","last_modified_at":"2013-08-12 08:00:20.457586991 +0200 CEST","recipients":["92EDE36B"],"version":"0.1.0"},"data":{}}


    # The data attached to the empty trousseau store are
    # the metadata. Fortunately trousseau exposes a meta
    # command to output them properly.
    $ trousseau meta
    CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
    LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
    Recipients: [4B7D890]
    TrousseauVersion: 0.1.0c
````

<div class="break"></div>
*Adding and removing recipients*

```bash
    # Now suppose you'd like another recipient, which
    # will then be able to open and update the trousseau store
    $ trousseau add-recipient 75FE3AB
    $ trousseau add-recipient 869FA4A
    $ trousseau meta
    CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
    LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
    Recipients: [4B7D890, 75FE3AB, 869FA4A]
    TrousseauVersion: 0.1.0c


    # And if you don't want to give your love anymore to some
    # of the store recipients, just remove him from the list
    $ trousseau remove-recipient 75FE3AB
    $ trousseau meta
    CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
    LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
    Recipients: [4B7D890, 869FA4A]
    TrousseauVersion: 0.1.0c
```

<div class="break"></div>
#### Getting, setting, deleting, listing keys

Once your trousseau has been created, you're now able to read, write, list, delete it's data and metadata. Here's how the fun part goes.

<div class="break"></div>
*Api*

* `get KEY` : Outputs the stored KEY-value pair
* `set KEY VALUE` : Sets the provided key-value pair in store
* `del KEY` : Deletes provided key from the store
* `keys` : Lists the stored keys
* `show` : Lists the stored key-value pairs

<div class="break"></div>
*Example*

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
#### Import/Export to remote storage

Trousseau was built with data remote storage in mind. As of today only S3 storage is available, but more are to come (don't forget to set your aws credentials environment variables)

<div class="break"></div>
*Api*

* `push` : Pushes the trousseau data store to remote storage
* `pull` : Pulls the trousseau data store from remote storage

<div class="break"></div>
*S3 Example*

Pushing the trousseau data store to Amazon S3 will require some setup:

* Make sure to set aws credentials environment variables

```bash
    $ export AWS_ACCESS_KEY_ID=myaeccskey
    $ export AWS_SECRET_ACCESS_KEY=mysecretkey
```

* You can setup the bucket to push data store into and the remote filename using environment. However, you're also able to provide these parameters as arguments of the **push** and **pull** methods.

```bash
    $ export TROUSSEAU_S3_FILENAME=trousseau
    $ export TROUSSEAU_S3_BUCKET=mytrousseaubucket
```

Once you've to set it up, you're ready to properly push the data store to S3.

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
*Scp example*

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

<div class="section-break"></div>
## More features to come
 
* Support for Sftp remote storage
* Support for GDrive remote storage
* Support for Dropbox remote storage

* In a further future I might had support for truecrypt encryption

<div class="section-break"></div>
## Contribute
 
* Check for open issues or open a fresh issue to start a discussion around a feature idea or a bug.
* Fork the repository on GitHub to start making your changes to the **master** branch (or branch off of it).
* Write tests which shows that the bug was fixed or that the feature works as expected.
* Send a pull request and bug the maintainer until it gets merged and published. :) Make sure to add yourself to AUTHORS.

