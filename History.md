
0.3.0 / 2013-04-21
==================

*User experience*
 * Add verbose flag
 * Fix #47 add a --file flag to get action
 * Fix #76 disable default data store truncate on open
 * Fix #75 prevent from removing the last recipient

*Code and design*
 * Enhance error reporting when public keys are missing
 * Implement a custom PgpError type to enhance encryption errors tracking
 * Allow gnupg keyring files to be selected via sys env
 * Simplify keyring and encryption/decryption actions definition
 * Rename keyring related openpgp args to be more obvious
 * Refactor decryption init to avoid global states
 * Refactor encryption init to avoid global states
 * Add goxc configuration file
 * Replace deprecated go.crypto/ssh package with gosshold/ssh
 * Implement verbosity option through commands
 * Add logrus logger in trousseau package

0.2.6 / 2014-04-17
==================

 * Fix #69 clean command-line output for its parsing to be easier
 * Fix #65 Add support for trousseau store selection through env
 * Fix #67 empty key field

0.2.5 / 2014-04-16
==================

 * Add a --file option to the set action

0.2.4 / 2014-02-26 
==================

 * Add gist remote storage usage instructions to README
 * Rename subcommands to commands
 * Move vagrant provisioning script into scripts folder
 * Refactor build to use both a simple Makefile and build script
 * Move trousseau package files in trousseau dir
 * Fix #1: Made error message more obvious when gnupg keyring cannot be opened
 * Remove gnupg globals from trousseau package
 * Throw fatal error when no passphrase
 * Enhance logging on missing passphrase or data store
 * Update error message when no passphrase are supplied
 * Fix #42: add the ability to declare recipients using their mail
 * Update openpgp encryption/decryption features naming
 * Fix #55: remove passphrase cmdline option
 * Update README scp example to use --ask-password [ref#59]

0.2.3 / 2014-02-18 
==================

 * Scp push/pull support for --ask-password option
 * Unified store encryption handling
 * Add encryption algorithm selection constants
 * Move encryption/decryption functionalities in the crypto package
 * Add openpgp encryption package

0.2.2 / 2014-02-12 
==================

 * Implement import command merging strategies

0.2.1 / 2014-02-12 
==================

 * Add: gist remote storage support
 * Fix: project dir linking to gopath
 * Add: an install rule to Makefile.
 * Merge branch 'hotfix/0.2_homebrew_formula' into develop
 * Fix homebrew formula

0.2.0 / 2013-12-02 
==================

New Features:
  * Support for osx and gnome keyring managers to provide the gpg master key passphrase
  * Support for gpg-agent to provide the gpg master key passphrase
  * Push/Pull operation now setup their destination and options through a DSN.
  * Support for s3 region option on push and pull actions
  * Adds a global --passphrase option to supply gpg master key passphrase
  * Adds Vagrantfile for easier trousseau dev environment setup
  * Adds a Homebrew formula for easier osx install

Bug fixes and refactoring:
  * Remote storage sources management was refactored
  * Ssh remote storage was fixed and now supports password authentication and passphrased keys


0.1.4 / 2013-11-26 
==================

 * Adds a global --passphrase option to supply gpg master key passphrase
 * Adds support for osx and gnome keyring managers to provide the gpg master key passphrase
 * Adds support for gpg-agent to provide the gpg master key passphrase
 * Adds support for s3 region option on push and pull actions
 * Adds Vagrantfile for easier trousseau dev environment setup
 * Set the default remote filename to trousseau.tsk 
 * Refactores Makefile GOPATH support
 * Refactores the push/pull options internals
 * Replaces the launchpad goamz aws wrapper with github.com/crowdmob/goamz
 * Rewords password to passphrase
 * Fix the error message when incorrect number of arguments passed.

0.1.3 / 2013-11-23
==================
* Add Trousseau boglio's logo
* Add package rule to Makefile
* Update refactoring Makefile
* Convert README to README
* Fix ssh.Publickey usage with rsa.Publickey in scp_storage
* Fix Makefile repository url
* Fixed Keychain ssh.ClientAuth interface implementation
* Removed binaries from the repository
* Fixed trousseau main package import

0.1.2 / 2013-09-04 
==================

 * Added import action
 * Added export action
 * Updated the error message when store file does not exist
 * Fixed gofmt
 * Update README.rst
