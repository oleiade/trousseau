# Create command
complete -f -c trousseau -n '__fish_complete_subcommand' -a create -d 'Create a trousseau data store'


# Meta command
complete -f -c trousseau -n '__fish_complete_subcommand' -a meta -d 'Show trousseau data store meta data'


# Get command
complete -f -c trousseau -n '__fish_complete_subcommand' -a get -d 'Get a key\'s value from the store'


# Set command
complete -f -c trousseau -n '__fish_complete_subcommand' -a set -d 'Set a store key-value pair'

# Rename command
complete -f -c trousseau -n '__fish_complete_subcommand' -a rename -d 'rename an existing key'

# Del command
complete -f -c trousseau -n '__fish_complete_subcommand' -a del -d 'Remove a key-value pair'

# Keys command
complete -f -c trousseau -n '__fish_complete_subcommand' -a keys -d 'Lists the store keys'

# Show command
complete -f -c trousseau -n '__fish_complete_subcommand' -a show -d 'shows trousseau content'

# Add-recipient command
complete -f -c trousseau -n '__fish_complete_subcommand' -a add-recipient -d 'add a recipient to the encrypted trousseau'

# Remove-recipient command
complete -f -c trousseau -n '__fish_complete_subcommand' -a remove-recipient -d 'remove a recipient of the encrypted trousseau'

# Import command
complete -f -c trousseau -n '__fish_complete_subcommand' -a import -d 'Import an encrypted trousseau file content' 
complete -f -c trousseau -n '__fish_complete_subcommand import' -l 'overwrite' -d 'Overwrite existing trousseau file'
complete -f -c trousseau -n '__fish_complete_subcommand import' -l 'theirs' -d 'Keep the imported file value'
complete -f -c trousseau -n '__fish_complete_subcommand import' -l 'yours' -d 'Keep your current data store values'


# Export command
complete -f -c trousseau -n '__fish_complete_subcommand' -a export -d 'Export the encrypted trousseau to local fs' 
complete -f -c trousseau -n '__fish_complete_subcommand export' -l 'overwrite' -d 'Overwrite existing trousseau file'


# Push command
complete -f -c trousseau -n '__fish_complete_subcommand' -a push -d 'Push the encrypted data store to remote storage' 
complete -f -c trousseau -n '__fish_complete_subcommand push' -l 'overwrite' -d 'Overwrite existing trousseau file' 
complete -f -c trousseau -n '__fish_complete_subcommand push' -l 'ask-password' -d 'Prompt for password' 
complete -f -c trousseau -n '__fish_complete_subcommand push' -l 'ssh-private-key' -d 'Path to the ssh private key to be used' 


# Pull command
complete -f -c trousseau -n '__fish_complete_subcommand' -a pull -d 'Push the encrypted data store to remote storage' 
complete -f -c trousseau -n '__fish_complete_subcommand pull' -l 'overwrite' -d 'Overwrite existing trousseau file' 
complete -f -c trousseau -n '__fish_complete_subcommand pull' -l 'ask-password' -d 'Prompt for password' 
complete -f -c trousseau -n '__fish_complete_subcommand pull' -l 'ssh-private-key' -d 'Path to the ssh private key to be used' 
