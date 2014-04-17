.. _setting_things_up:

=================
Setting things up
=================

.. _installation:

Installation
============

Debian and ubuntu
-----------------

A binary debian repository provides *trousseau* packages for *i386*, *x86_64* and *arm* architectures, so you can easily install it.
Just add the repository to your sources.list:

.. code-block:: bash

    $ echo "deb http://dl.bintray.com/oleiade/deb /" | sudo tee /etc/apt/sources.list.d/trousseau.list

And you're ready to go:

.. code-block:: bash

    $ sudo apt-get update && sudo apt-get install trousseau

OSX
---

Homebrew
~~~~~~~~

If you're using homebrew just proceed to installation using the provided formula:

.. code-block:: bash

    $ brew install trousseau.rb

*Et voila!*

Macports
~~~~~~~~

Coming soon (Don't be shy, if you feel like you could do it, just send pull request ;) )

Build it
~~~~~~~~

1. First, make sure you have a `Go <http://http://golang.org/>`_ language compiler **>= 1.1.2** (*mandatory*) and `git <http://gitscm.org>`_ installed.
2. Make sure you have the following go system dependencies in your ``$PATH``: `bzr, svn, hg, git`
3. Then, just build and copy the ``./bin/trousseau`` executable to a system *PATH* location

.. code-block:: bash

    make
    sudo cp ./bin/trousseau /usr/local/bin/trousseau


.. _prerequisities:

Prerequisities
==============

Gpg passphrase
--------------

Every decryption operations will require your *gpg* primary key passphrase.

As of today, **trousseau** is able to handle your passphrase through multiple ways:

* system's keyring manager
* gpg-agent daemon
* system environment
* ``--passphrase`` global option

Keyring manager
~~~~~~~~~~~~~~~

Supported system keyring manager are osx keychain access and linux gnome secret-service and gnome-keychain (more might be added in the future on demand).
To use the keyring manager you will need to set up the ``TROUSSEAU_KEYRING_SERVICE`` environment variable to the name of they keyring manager key holding the trousseau main gpg key passphrase.

.. code-block:: bash

    $ export TROUSSEAU_KEYRING_SERVICE=my_keyring_key
    $ trousseau get abc

Gpg agent
~~~~~~~~~

Another authentication method supported is gpg-agent. In order to use it make sure you've started the gpg-agent daemon and exported the ``GPG_AGENT_INFO`` variable, trousseau will do the rest.

.. code-block:: bash

    $ export GPG_AGENT_INFO=path_to_the_gpg_agent_info_file
    $ export TROUSSEAU_MASTER_GPG_ID=myid@mymail.com
    $ trousseau get abc

Environment variable
~~~~~~~~~~~~~~~~~~~~

Alternatively, you can pass your primary key passphrase as ``TROUSSEAU_PASSPHRASE`` environment variable:

.. code-block:: bash

    $ export TROUSSEAU_PASSPHRASE=mysupperdupperpassphrase
    $ trousseau get abc

Passphrase global option
~~~~~~~~~~~~~~~~~~~~~~~~

Ultimately, you can pass you gpg passphrase through the command line global option:

.. code-block:: bash

    $ trousseau --passhphrase mysupperdupperpassphrase get abc

Environment
-----------

Trousseau behavior can be controlled through the system environment:

* *TROUSSEAU_STORE* : if you want to have multiple trousseau data store, set this environment variable to the path of the one you want to use. Default is ``$HOME/.trousseau``
