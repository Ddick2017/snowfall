#!/usr/bin/env bash
# Installs the binary from the .tar.gz into ~/.local/bin and enables autostart.
set -e
mkdir -p "$HOME/.local/bin"
install -m 755 "$(dirname "$0")/snowfall" "$HOME/.local/bin/snowfall"
"$HOME/.local/bin/snowfall" --autostart-on
echo "Done. Start now with:  nohup ~/.local/bin/snowfall >/dev/null 2>&1 &"
echo "Stop with:             snowfall --quit"
