@echo off
rem Fake $EDITOR/$VISUAL for `trousseau-cli`'s `edit` integration tests
rem (crates\trousseau-cli\tests\edit.rs) on Windows. See fake-editor.sh
rem for the shared TEST_EDIT_ACTION/TEST_EDIT_PATH_OUT/TEST_EDIT_MODE_OUT
rem contract; table-block edits (`remove`, `change`) shell out to
rem PowerShell since batch has no adequate text-processing tool.
setlocal enabledelayedexpansion

set "scratch=%~1"
set "action=%TEST_EDIT_ACTION%"
if "%action%"=="" set "action=noop"

if not "%TEST_EDIT_PATH_OUT%"=="" (
    > "%TEST_EDIT_PATH_OUT%" (set /p "=%scratch%")
)

if not "%TEST_EDIT_MODE_OUT%"=="" (
    rem Windows has no POSIX permission bits; the mode test is Unix-only
    rem (crates\trousseau-cli\tests\edit.rs), so this branch is never
    rem asserted on. Write a placeholder so the side file still exists.
    > "%TEST_EDIT_MODE_OUT%" echo 600
)

if /I "%action%"=="noop" goto :eof

if /I "%action%"=="empty" (
    type nul > "%scratch%"
    goto :eof
)

if /I "%action%"=="add" (
    powershell -NoProfile -Command "Add-Content -NoNewline -Path '%scratch%' -Value \"`n[`\"added/key`\"]`nvalue = `\"added-value`\"`n\""
    goto :eof
)

if /I "%action%"=="remove" (
    powershell -NoProfile -Command "$c = Get-Content -Raw -Path '%scratch%'; $c = [regex]::Replace($c, '(?ms)^\[\"a/removeme\"\].*?(\r?\n\r?\n|\z)', ''); Set-Content -NoNewline -Path '%scratch%' -Value $c"
    goto :eof
)

if /I "%action%"=="change" (
    powershell -NoProfile -Command "$c = Get-Content -Raw -Path '%scratch%'; $c = [regex]::Replace($c, '(?ms)(^\[\"a/changeme\"\].*?^value = ).*$', '${1}\"changed\"', 1); Set-Content -NoNewline -Path '%scratch%' -Value $c"
    goto :eof
)

if /I "%action%"=="corrupt" (
    > "%scratch%" echo this is not valid toml [[[
    goto :eof
)

if /I "%action%"=="fail" (
    exit /b 3
)

echo fake-editor.cmd: unknown TEST_EDIT_ACTION: %action% 1>&2
exit /b 1
