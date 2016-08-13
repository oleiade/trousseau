# Change Log

## [Unreleased](https://github.com/oleiade/trousseau/tree/HEAD)

[Full Changelog](https://github.com/oleiade/trousseau/compare/0.3.6...HEAD)

**Fixed bugs:**

- package not found while compiling [\#173](https://github.com/oleiade/trousseau/issues/173)

**Closed issues:**

- Debian jessie apt install problem [\#172](https://github.com/oleiade/trousseau/issues/172)

## [0.3.6](https://github.com/oleiade/trousseau/tree/0.3.6) (2016-08-12)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.3.4...0.3.6)

**Fixed bugs:**

- code.google.com/p/goauth2/oauth is deprecated [\#174](https://github.com/oleiade/trousseau/issues/174)
- Creating store with symmetric encryption without supplying passphrase should not be possible [\#140](https://github.com/oleiade/trousseau/issues/140)
- Show command adds \n once in a while [\#138](https://github.com/oleiade/trousseau/issues/138)
- trousseau store has +x set \(0.3.2\) [\#118](https://github.com/oleiade/trousseau/issues/118)

**Closed issues:**

- code.google.com/archive/p/gosshold/ssh is deprecated [\#176](https://github.com/oleiade/trousseau/issues/176)
- Take set value from stdin [\#136](https://github.com/oleiade/trousseau/issues/136)
- Actions should not log, they should return errors to be logged [\#132](https://github.com/oleiade/trousseau/issues/132)
- Add crypto selection related options to the create command [\#131](https://github.com/oleiade/trousseau/issues/131)
- Explicit the error message when wrong passphrase provided to ask-passphrase option [\#130](https://github.com/oleiade/trousseau/issues/130)
- Refactor the ask-passphrase option behavior to be more explicit [\#129](https://github.com/oleiade/trousseau/issues/129)
- export/import commands should echo on stdout/read on stdin when no arguments are provided  [\#125](https://github.com/oleiade/trousseau/issues/125)
- Implement a clear file detection strategy [\#113](https://github.com/oleiade/trousseau/issues/113)
- Unify push/pull and import/export commands as a single import/export command pair [\#108](https://github.com/oleiade/trousseau/issues/108)
- Add a configure command [\#107](https://github.com/oleiade/trousseau/issues/107)
- trousseau push/pull commands should support named endpoints [\#106](https://github.com/oleiade/trousseau/issues/106)

## [0.3.4](https://github.com/oleiade/trousseau/tree/0.3.4) (2014-10-06)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.3.3...0.3.4)

**Fixed bugs:**

- 118 filemode [\#120](https://github.com/oleiade/trousseau/pull/120) ([jd1123](https://github.com/jd1123))

**Closed issues:**

- create command with no recipents causes an out of bounds runtime error [\#122](https://github.com/oleiade/trousseau/issues/122)

## [0.3.3](https://github.com/oleiade/trousseau/tree/0.3.3) (2014-09-22)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.3.2...0.3.3)

**Fixed bugs:**

- Find out how to fulfill properly the debian package meta data fields [\#117](https://github.com/oleiade/trousseau/issues/117)

**Closed issues:**

- `trousseau create` fails for v0.3.2 [\#116](https://github.com/oleiade/trousseau/issues/116)
- "trousseau set" --file paramter is broken in 0.3.2 [\#115](https://github.com/oleiade/trousseau/issues/115)
- Error when updating via apt [\#114](https://github.com/oleiade/trousseau/issues/114)

## [0.3.2](https://github.com/oleiade/trousseau/tree/0.3.2) (2014-09-15)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.3.1...0.3.2)

**Fixed bugs:**

- --store option does not override environment [\#112](https://github.com/oleiade/trousseau/issues/112)
- Irrational logging of upgrade command [\#104](https://github.com/oleiade/trousseau/issues/104)
- trousseau create with multiple recipients raises a 'configure' error [\#95](https://github.com/oleiade/trousseau/issues/95)
- Data store created even though there was an error [\#89](https://github.com/oleiade/trousseau/issues/89)
- Import when trousseau store does not exists raises an error [\#58](https://github.com/oleiade/trousseau/issues/58)

**Closed issues:**

- Add an exists command [\#92](https://github.com/oleiade/trousseau/issues/92)
- Explicit errors when no compatible private key was found to open the data store [\#111](https://github.com/oleiade/trousseau/issues/111)
- Enhance error message when trousseau is unable to open old-version trousseau data store [\#109](https://github.com/oleiade/trousseau/issues/109)
- Support alternative gnupg home directory [\#103](https://github.com/oleiade/trousseau/issues/103)
- Reduce global usage variables [\#101](https://github.com/oleiade/trousseau/issues/101)

## [0.3.1](https://github.com/oleiade/trousseau/tree/0.3.1) (2014-09-11)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.3.0...0.3.1)

**Fixed bugs:**

- Behavior when renaming a key should be configurable [\#90](https://github.com/oleiade/trousseau/issues/90)

**Closed issues:**

- Cleanup logging [\#100](https://github.com/oleiade/trousseau/issues/100)
- Change file format to prepare Symmetric PGP and AES256 support [\#97](https://github.com/oleiade/trousseau/issues/97)
- Add a list-recipients command [\#91](https://github.com/oleiade/trousseau/issues/91)
- Add a --store option and a select-store  command to select store to use [\#85](https://github.com/oleiade/trousseau/issues/85)

## [0.3.0](https://github.com/oleiade/trousseau/tree/0.3.0) (2014-04-22)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.6...0.3.0)

**Fixed bugs:**

- Add recipient with invalid recipient corrupts data store [\#76](https://github.com/oleiade/trousseau/issues/76)
- Prevent from removing the last recipient [\#75](https://github.com/oleiade/trousseau/issues/75)
- Once passphrase is extracted, it appears in trousseau help [\#55](https://github.com/oleiade/trousseau/issues/55)
- Import command should "really" import pointed file content [\#54](https://github.com/oleiade/trousseau/issues/54)
- Add import merging strategies [\#53](https://github.com/oleiade/trousseau/issues/53)
- On linux trousseau tries to automatically talk to keyring manager [\#48](https://github.com/oleiade/trousseau/issues/48)
- Import of a trousseau works even without gpg passphrase being submitted [\#46](https://github.com/oleiade/trousseau/issues/46)
- Raise an error when a push/pull scheme is unhandled [\#40](https://github.com/oleiade/trousseau/issues/40)

**Closed issues:**

- Add a --verbose mode [\#72](https://github.com/oleiade/trousseau/issues/72)
- Clean the trousseau stdout output to be easily parsed [\#69](https://github.com/oleiade/trousseau/issues/69)
- Trousseau 0.5 always set an empty key \(and is therefore completely fucked up\) [\#67](https://github.com/oleiade/trousseau/issues/67)
- Support for alternative TROUSSEAU\_HOME [\#65](https://github.com/oleiade/trousseau/issues/65)
- Add a --file flag to the set action [\#63](https://github.com/oleiade/trousseau/issues/63)
- Prompt for password [\#60](https://github.com/oleiade/trousseau/issues/60)
- Dropping credentials into your shell history is bad news. [\#59](https://github.com/oleiade/trousseau/issues/59)
- Ability to export a given list of keys values to a file [\#47](https://github.com/oleiade/trousseau/issues/47)
- Ability to declare recipients using their mail [\#42](https://github.com/oleiade/trousseau/issues/42)
- Support Gist as remote destination [\#37](https://github.com/oleiade/trousseau/issues/37)
- Allow for multiple Trousseau stores [\#10](https://github.com/oleiade/trousseau/issues/10)
- Import command should try to merge the imported file content into it's current store [\#4](https://github.com/oleiade/trousseau/issues/4)
- Error when gpg public key is not available error mesages should be more obvious [\#2](https://github.com/oleiade/trousseau/issues/2)
- Instructions when pub or sec rings are not created [\#1](https://github.com/oleiade/trousseau/issues/1)

**Merged pull requests:**

- Fix a bug where the key is always empty when set a key. [\#68](https://github.com/oleiade/trousseau/pull/68) ([fdv](https://github.com/fdv))

## [0.2.6](https://github.com/oleiade/trousseau/tree/0.2.6) (2014-04-17)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.5...0.2.6)

## [0.2.5](https://github.com/oleiade/trousseau/tree/0.2.5) (2014-04-17)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.4...0.2.5)

## [0.2.4](https://github.com/oleiade/trousseau/tree/0.2.4) (2014-03-05)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.3...0.2.4)

## [0.2.3](https://github.com/oleiade/trousseau/tree/0.2.3) (2014-02-18)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.2...0.2.3)

## [0.2.2](https://github.com/oleiade/trousseau/tree/0.2.2) (2014-02-12)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.1...0.2.2)

## [0.2.1](https://github.com/oleiade/trousseau/tree/0.2.1) (2014-02-12)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.2.0...0.2.1)

## [0.2.0](https://github.com/oleiade/trousseau/tree/0.2.0) (2013-12-02)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.1.4...0.2.0)

**Fixed bugs:**

- Automate dsn default value selection [\#29](https://github.com/oleiade/trousseau/issues/29)
- Make sure the --password option is taken in account [\#13](https://github.com/oleiade/trousseau/issues/13)
- make tries to mkdir in system dirs [\#5](https://github.com/oleiade/trousseau/issues/5)

**Closed issues:**

- 0.2.0 Release todo list [\#36](https://github.com/oleiade/trousseau/issues/36)
- Add a homebrew recipe [\#34](https://github.com/oleiade/trousseau/issues/34)
- Support both hostname and ip adress as scp host [\#30](https://github.com/oleiade/trousseau/issues/30)
- Add dsn submodule tests [\#28](https://github.com/oleiade/trousseau/issues/28)
- Support scp password authentication [\#27](https://github.com/oleiade/trousseau/issues/27)
- Support gpg-agent as a gpg password alternative [\#18](https://github.com/oleiade/trousseau/issues/18)
- Specify push/pull destinations with URL rather than flags [\#17](https://github.com/oleiade/trousseau/issues/17)
- Implement tmc/keyring as a password supply alternative [\#14](https://github.com/oleiade/trousseau/issues/14)
- Do not take password from environnment [\#12](https://github.com/oleiade/trousseau/issues/12)

## [0.1.4](https://github.com/oleiade/trousseau/tree/0.1.4) (2013-11-26)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.1.3...0.1.4)

## [0.1.3](https://github.com/oleiade/trousseau/tree/0.1.3) (2013-11-23)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.1.2...0.1.3)

## [0.1.2](https://github.com/oleiade/trousseau/tree/0.1.2) (2013-09-04)
[Full Changelog](https://github.com/oleiade/trousseau/compare/0.1.1...0.1.2)

## [0.1.1](https://github.com/oleiade/trousseau/tree/0.1.1) (2013-08-15)

0.3.4 / 2014-10-06
==================

 * Fix #119: add missing errors return in trousseau's openpgp package
 * Fix #121: Ensure files are created in 0600 mode
 * Enhance integration testing
 * Add dummy gpg keys for testing purposes
 * Fix create store for multiple recipients

0.3.3 / 2014-09-22
==================

 * Fix #117: fill the debian packages metadata Description field
 * Fix #116: raise and error when no recipients were provided to the create function
 * Fix #115 expected args count whether --file option is passed or not and add tests
 * Add integration tests with bats

0.3.2 / 2014-09-15
==================

 * Fix store path evaluation order option > env > default [fix #112]
 * Add support for alternative gnupg home [fix #103]
 * Generate gnupg pubring and secring at execution time [ref #103]
 * trousseau/crypto/openpgp cleanup and enhancements
 * Remove globals.go file [ref #101]
 * Enhance logging when no private key able to decrypt data store found [fix #111]
 * Raise a proper error message when outdated data store format is detected [fix #109]
 * Remove useless logging from upgrade command [fix #104]
 * Fix import raises an error when data store does not exist [fix #58]
 * Support for multiple recipients on data store creation [fix #95]
 * Throw error when recipient does not exist on create command [fix #89]

0.3.1 / 2014-09-10
==================

!! Backward Incompatibility !!

Trousseau data store file format changed, and trousseau >= 0.3.1 are 
now incompatible with older version created files.

Fortunately, trousseau now exposes a 'upgrade' command which will take
care to upgrade your existing data stores.

So if you are upgrading from former versions, please, upgrade.

*Features and user experience* 
 * New data store file format: support for different encryption type and algorithms. Plain and Encrypted sections splitted.
 * New upgrade command to automatically upgrade old versions data store to new format.
 * Added a rename command to modify a key name
 * Added a list-recipients command  to easily show data store recipients
 * Added a --store global option to select directly from command line data store to be used
 * Added bash, zsh, and fish autocompletion rules in scripts/
 * Updated import and export commands to support plain data import/export through a --plain option
 * Updated trousseau keys and show commands output so they are now alphabetically sorted
 * Fixed trousseau command piped output 
 * Fixed trousseau dependency management reliability through godep
 * Improved command-line accessibility: more obvious behaviors, commands and flags descriptions
 * Improved Makefile


*Code and design*
 * Reduce inter-dependency between trousseau package and cli interactions
 * Moved command actions in trousseau package, got rid of cli.Context dependency.
 * Replaced (trousseau)cli package with idiomatic cmd/trousseau/*
 * Got rid of a ton of useless abstractions. More to go.
 * Removed logrus dependency and use stdlib log package instead
 * Rename GetStorePath to InferStorePath and add getters/setters on the gStorePath global
 * Rename upload* helpers to Helper*
 * Move S3 and Scp defaults globals to context.go
 * Add a store file path retrieval helper
 * Move passphrase handling in context.go
 * Remove global passphrase + use getter in cli instead
 * Copy the cli interface trousseau package members to a new cli package


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


\* *This Change Log was automatically generated by [github_changelog_generator](https://github.com/skywinder/Github-Changelog-Generator)*