#!/usr/bin/env bash
#
# Magma Terminal — Uninstaller
#
# Removes the binary, desktop entry, and icon installed by install.sh.
# Optionally removes user configuration data.
#
# Usage:
#   ./uninstall.sh          # remove app only
#   ./uninstall.sh --purge  # also remove config and setup data
#

set -euo pipefail

APP_NAME="magma"
DESKTOP_ID="io.magma.terminal"

BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/64x64/apps"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/magma"

PURGE=false
if [[ "${1:-}" == "--purge" ]]; then
  PURGE=true
fi

remove_file() {
  local path="$1"
  local label="$2"
  if [[ -f "$path" ]]; then
    rm "$path"
    echo "  removed $label → $path"
  else
    echo "  skip    $label (not found)"
  fi
}

echo "Uninstalling Magma Terminal..."
echo ""

remove_file "$BIN_DIR/$APP_NAME" "binary"
remove_file "$APPS_DIR/$DESKTOP_ID.desktop" "desktop"
remove_file "$ICON_DIR/$DESKTOP_ID.png" "icon"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

if $PURGE; then
  echo ""
  if [[ -d "$CONFIG_DIR" ]]; then
    rm -rf "$CONFIG_DIR"
    echo "  purged  config → $CONFIG_DIR"
  else
    echo "  skip    config (not found)"
  fi
fi

echo ""
echo "Magma has been uninstalled."
if ! $PURGE; then
  echo "Your configuration in $CONFIG_DIR was kept."
  echo "Run with --purge to also remove config data."
fi
