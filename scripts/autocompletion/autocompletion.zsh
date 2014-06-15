## Copyright (c) 2014 Chris Aumann
##
## See the file LICENSE for copying permission.

#compdef trousseau

_trousseau() {
  local curcontext="$curcontext" state line
  typeset -A opt_args

  _arguments '1: :->trousseaucmd' \
             '2: :->trousseausubcmd'

  case $state in
    trousseaucmd)
      # List of available commands
      compadd -Q create push pull export import add-recipient remove-recipient set get del keys show meta help
    ;;
    trousseausubcmd)
      case $words[2] in
        (get|set|del)
          # Retrieve get/set autocomplete from "trousseau keys"
          keys=("${(@f)$(trousseau keys)}")
          compadd -a "$@" -- keys
        ;;
        *)
          # Fallback to standard filename completion
          compadd - *
        ;;
      esac
    ;;
    *)
      # Fallback to standard filename completion
      compadd - *
    ;;
  esac
}

_trousseau "$@"

