
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
