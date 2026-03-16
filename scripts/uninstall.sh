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
LEGACY_APP_NAME="obsidian"
LEGACY_DESKTOP_ID="io.obsidian.terminal"

BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICON_ROOT="$HOME/.local/share/icons/hicolor"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/magma"
LEGACY_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/obsidian"
APP_DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/$DESKTOP_ID"
LEGACY_APP_DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/$LEGACY_DESKTOP_ID"
LEGACY_SHARE_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/obsidian"

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

remove_icons() {
  local desktop_id="$1"
  local removed=false
  while IFS= read -r path; do
    rm "$path"
    echo "  removed icon   → $path"
    removed=true
  done < <(find "$ICON_ROOT" -path "*/apps/$desktop_id.png" -print 2>/dev/null)

  if [[ "$removed" == false ]]; then
    echo "  skip    icon (not found for $desktop_id)"
  fi
}

echo "Uninstalling Magma Terminal..."
echo ""

remove_file "$BIN_DIR/$APP_NAME" "binary"
remove_file "$APPS_DIR/$DESKTOP_ID.desktop" "desktop"
remove_icons "$DESKTOP_ID"
remove_file "$BIN_DIR/$LEGACY_APP_NAME" "legacy binary"
remove_file "$APPS_DIR/$LEGACY_DESKTOP_ID.desktop" "legacy desktop"
remove_icons "$LEGACY_DESKTOP_ID"

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

  if [[ -d "$LEGACY_CONFIG_DIR" ]]; then
    rm -rf "$LEGACY_CONFIG_DIR"
    echo "  purged  legacy config → $LEGACY_CONFIG_DIR"
  else
    echo "  skip    legacy config (not found)"
  fi

  if [[ -d "$APP_DATA_DIR" ]]; then
    rm -rf "$APP_DATA_DIR"
    echo "  purged  data → $APP_DATA_DIR"
  else
    echo "  skip    data (not found)"
  fi

  if [[ -d "$LEGACY_APP_DATA_DIR" ]]; then
    rm -rf "$LEGACY_APP_DATA_DIR"
    echo "  purged  legacy data → $LEGACY_APP_DATA_DIR"
  else
    echo "  skip    legacy data (not found)"
  fi

  if [[ -d "$LEGACY_SHARE_DIR" ]]; then
    rm -rf "$LEGACY_SHARE_DIR"
    echo "  purged  legacy share → $LEGACY_SHARE_DIR"
  else
    echo "  skip    legacy share (not found)"
  fi
fi

echo ""
echo "Magma has been uninstalled."
if ! $PURGE; then
  echo "Your configuration in $CONFIG_DIR was kept."
  echo "Run with --purge to also remove config and legacy data."
fi
