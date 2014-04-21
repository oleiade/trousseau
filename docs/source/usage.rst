.. _usage:

=====
Usage
=====

.. _store_creation:

Store creation
==============

First use of **trousseau** requires the data store to be created. A **trousseau** data store is built and maintained for a list of *gpg* recipients who will be the only ones able to decrypt and manipulate it (so don't forget to include yourself ;) )

API
---

* **create** [RECIPIENTS ...] : creates the trousseau encrypted datastore for provided recipients and stores it in ``$HOME/.trousseau``
* **meta** : Outputs the store metadata.
* **add-recipient** RECIPIENT : Adds a recipient to the store. The recipient will be able to open and modify the store.
* **remove-recipient** RECIPIENT : Removes a recipient from the store. The recipient will not be able to open or modify the store.

First steps with the data store
-------------------------------

.. code-block:: bash

    $ trousseau create 4B7D890,28EA78B  # create a trousseau for two gpg recipients
    trousseau created at $HOME/.trousseau

Trousseau data store consists in single gpg encrypted file residing in your ``$HOME`` directory. Check by yourself.

.. code-block:: bash

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

.. _keys_manipulation:

Keys manipulation
=================

Once your trousseau has been created, you're now able to read, write, list, delete its data. Here's how the fun part goes.

API
---

* **get** KEY [--file]: Outputs the stored KEY-value pair, whether on *stdout* or in pointed ``--file`` option path.
* **set** KEY [VALUE | --file] : Sets the provided key-value pair in store using provided value or extracting it from path pointed by ``--file`` option.
* **del** KEY : Deletes provided key from the store
* **keys** : Lists the stored keys
* **show** : Lists the stored key-value pairs

You've got the keys
-------------------

.. code-block:: bash

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
    myuser.ssh.public_key --file ~/.ssh/id_rsa.pub

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


.. _remote_import_export:

Remote storage import/export
============================

Trousseau was built with data remote storage in mind. Therefore it provides *push* and *pull* actions to export and import the trousseau data store to remote destinations.
As of today S3 and SSH storages are available (more are to come).
Moreover, 

API
---

* **push** : Pushes the trousseau data store to remote storage
* **pull** : Pulls the trousseau data store from remote storage

DSN
---

In order to make your life easier trousseau allows you to select your export and import sources using a *DSN*.

.. code-block::

    {protocol}://{identifier}:{secret}@{host}:{port}/{path}

* **protocol**: The remote service target type. Can be one of: *s3* or *scp*
* **identifier**: The login/key/whatever to authenticate **trousseau** to the remote service. Provide your *aws_access_key* if you're targeting *s3*, or your remote login if you're targeting *scp*.
* **secret**: The secret to authenticate **trousseau** to the remote service. Provide your *aws_secret_key* if you're targeting *s3*, or your remote password if you're targeting *scp*.
* **host**: Your bucket name is you're targeting *s3*. The host to login to using *scp* otherwise.
* **port**: The *aws_region* if you're targeting *s3*. The port to login to using *scp* otherwise.
* **path**: The remote path to push to or retrieve from the trousseau file on a ``push`` or ``pull`` action.

S3 Example
----------

.. code-block:: bash

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

Scp example
-----------

.. code-block:: bash

    # We start with a non-empty trousseau data store
    $ trousseau show
    abc: 123
    easy as: do re mi

    # To push it using scp we need to provide it a couple of
    # basic options
    $ trousseau push scp://user:password@host:port/remote_file_path


    # Now that data store has been pushed to the remote storage
    # using scp, let's remove the local data store and pull it
    # once again to ensure it worked
    $ rm ~/.trousseau
    $ trousseau show
    Trousseau unconfigured: no data store

    $ trousseau pull scp://user:password@host:port/remote_file_path
    $ trousseau show
    abc: 123
    easy as: do re mi

.. _local_import_export:

Local imports and exports
=========================

API
---

* **import** FILENAME: will import a trousseau data store from the local filesystem. The operation **erases** the current trousseau store content.
* **export** FILENAME: will export the current trousseau data store as `FILENAME` on the local fs.

Real world example
------------------

.. code-block:: bash

    $ trousseau export testtrousseau.asc  # Fine we've exported our current data store into a single file
    $ mail -f testtrousseau.asc cousin@machin.com  # Let's pretend we've sent it by mail

    # Now cousin machin is now able to import the data store
    $ trousseau import testtrousseau.asc
    $ trousseau show
    cousin_machin:isagreatbuddy
    adams_family:rests in peace, for sure

.. _metadata:

Metadata
========

Trousseau keeps track and exposes all sort of metadata about your store that you can access through the ``meta`` command.

.. code-block:: bash

    $ trousseau meta
    CreatedAt: 2013-08-12 08:00:20.457477714 +0200 CEST
    LastModifiedAt: 2013-08-12 08:00:20.457586991 +0200 CEST
    Recipients: [4B7D890,28EA78B]
    TrousseauVersion: 0.1.0c

Once again, if you're intersted in how the meta data are stored, go check yourself by decrypting the store content using one of your recipients private key.

.. code-block:: bash

    $ cat ~/.trousseau | gpg -d -r 4B7D890 --textmode
    You need a passphrase to unlock the secret key for
    user: "My Gpg User <MyGpg@mail.com>"
    2048-bit RSA key, ID 4B7D890, created 2013-05-21 (main key ID 4B7D890)

    gpg: encrypted with 2048-bit RSA key, ID 4B7D890, created 2013-05-21
    "My Gpg User <MyGpg@mail.com>"
    {"_meta":{"created_at":"2013-08-12 08:00:20.457477714 +0200 CEST","last_modified_at":"2013-08-12 08:00:20.457586991 +0200 CEST","recipients":["92EDE36B"],"version":"0.1.0"},"data":{}}

Adding and removing recipients
------------------------------

Okay, so you've created a trousseau data store with two recipients allowed to manipulate it. Now suppose you'd like to add another recipient to be able to open and update the trousseau store; or to remove one.
``add-recipient`` and ``remove-recipient`` commands can help you with that.

.. code-block:: bash

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

