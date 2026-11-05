#!/usr/bin/env bash
# Installs the TaskBoard Linux client.
#
#   ./install.sh          → current user only: ~/.local (no sudo needed)
#   sudo ./install.sh     → system-wide: /usr/local
set -euo pipefail

SYSTEM=0
if [ "$(id -u)" = 0 ] || [ "${1:-}" = "--system" ]; then
  SYSTEM=1
fi

if [ "$SYSTEM" = 1 ]; then
  BIN_DIR=/usr/local/bin
  APP_DIR=/usr/local/share/applications
  ICON_DIR=/usr/local/share/icons/hicolor/scalable/apps
else
  BIN_DIR="$HOME/.local/bin"
  APP_DIR="$HOME/.local/share/applications"
  ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"
install -m 755 "$SCRIPT_DIR/taskboard" "$BIN_DIR/taskboard"
install -m 644 "$SCRIPT_DIR/taskboard.svg" "$ICON_DIR/taskboard.svg"

# Desktop entry with the real binary path (the shipped file is a template)
sed "s|Exec=/usr/bin/taskboard|Exec=$BIN_DIR/taskboard|" \
  "$SCRIPT_DIR/taskboard.desktop" > "$APP_DIR/taskboard.desktop"

echo "Installed:"
echo "  $BIN_DIR/taskboard"
echo "  $APP_DIR/taskboard.desktop"
if [ "$SYSTEM" = 0 ]; then
  echo "Make sure ~/.local/bin is on your PATH, then launch 'taskboard'"
  echo "from your application menu or terminal."
fi
