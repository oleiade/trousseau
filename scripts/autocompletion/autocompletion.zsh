#compdef trousseau
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#

_trousseau() {
  local curcontext="$curcontext" state line
  typeset -A opt_args

  _arguments '1: :->trousseaucmd' \
             '2: :->trousseausubcmd'

  case $state in
    trousseaucmd)
      # List of available commands
      compadd -Q create push pull export import add-recipient remove-recipient set get rename del keys show meta help
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

