#!/bin/sh
# Fake $EDITOR/$VISUAL for `trousseau-cli`'s `edit` integration tests
# (crates/trousseau-cli/tests/edit.rs). Trousseau invokes it as
# `fake-editor.sh SCRATCH_PATH`; it rewrites SCRATCH_PATH according to
# TEST_EDIT_ACTION, one of: noop, empty, add, remove, change, corrupt,
# fail. `remove` and `change` operate on the fixed keys `a/removeme` and
# `a/changeme` the test that uses them sets up.
#
# Two optional side channels let a test observe things `trousseau edit`
# itself never prints: TEST_EDIT_PATH_OUT receives the scratch file's
# path (so a test can assert it is gone once `trousseau edit` returns),
# and TEST_EDIT_MODE_OUT receives its permission bits (octal, no leading
# zero) before this script's action runs.
set -eu

scratch="$1"

if [ -n "${TEST_EDIT_PATH_OUT:-}" ]; then
    printf '%s' "$scratch" > "$TEST_EDIT_PATH_OUT"
fi

if [ -n "${TEST_EDIT_MODE_OUT:-}" ]; then
    mode=$(stat -c '%a' "$scratch" 2>/dev/null || stat -f '%Lp' "$scratch")
    printf '%s' "$mode" > "$TEST_EDIT_MODE_OUT"
fi

action="${TEST_EDIT_ACTION:-noop}"

case "$action" in
    noop)
        # Leave the scratch file exactly as `trousseau edit` wrote it.
        ;;
    empty)
        : > "$scratch"
        ;;
    add)
        printf '\n["added/key"]\nvalue = "added-value"\n' >> "$scratch"
        ;;
    remove)
        # Delete the `["a/removeme"]` table: from its header line
        # through the following blank line (the separator
        # `toml::to_string_pretty` puts between tables).
        sed '\#^\["a/removeme"\]#,/^$/d' "$scratch" > "$scratch.tmp"
        mv "$scratch.tmp" "$scratch"
        ;;
    change)
        # Rewrite the `value` line inside the `["a/changeme"]` table
        # only.
        sed -E '\#^\["a/changeme"\]#,/^$/ s/^value = .*/value = "changed"/' "$scratch" > "$scratch.tmp"
        mv "$scratch.tmp" "$scratch"
        ;;
    corrupt)
        printf 'this is not valid toml [[[\n' > "$scratch"
        ;;
    fail)
        exit 3
        ;;
    *)
        echo "fake-editor.sh: unknown TEST_EDIT_ACTION: $action" >&2
        exit 1
        ;;
esac
