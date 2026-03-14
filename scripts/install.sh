#!/usr/bin/env bash
#
# Obsidian Terminal — Installer
#
# Installs the Obsidian AppImage, .desktop entry, and icon so the
# application is launchable from both the terminal and app menus.
#
# Usage:
#   ./install.sh                    # install from release tarball
#   ./install.sh /path/to/AppImage  # install a standalone AppImage
#

set -euo pipefail

APP_NAME="obsidian"
DESKTOP_ID="io.obsidian.terminal"
APPIMAGE_NAME="Obsidian-x86_64.AppImage"

BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/64x64/apps"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ---------------------------------------------------------------------------
# Locate the AppImage
# ---------------------------------------------------------------------------
find_appimage() {
  if [[ -n "${1:-}" && -f "$1" ]]; then
    echo "$1"
    return
  fi

  # When run from inside the release tarball directory
  if [[ -f "$SCRIPT_DIR/$APPIMAGE_NAME" ]]; then
    echo "$SCRIPT_DIR/$APPIMAGE_NAME"
    return
  fi

  echo ""
}

# ---------------------------------------------------------------------------
# Locate assets (icon + desktop file)
# ---------------------------------------------------------------------------
find_assets_dir() {
  # Release tarball layout: obsidian-release/assets/
  if [[ -d "$SCRIPT_DIR/assets" ]]; then
    echo "$SCRIPT_DIR/assets"
    return
  fi

  # Running from the repo: scripts/../assets
  local repo_assets="$SCRIPT_DIR/../assets"
  if [[ -d "$repo_assets" ]]; then
    echo "$(cd "$repo_assets" && pwd)"
    return
  fi

  echo ""
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
  local appimage
  appimage="$(find_appimage "${1:-}")"

  if [[ -z "$appimage" ]]; then
    echo "Error: could not find $APPIMAGE_NAME." >&2
    echo "Run this script from the release directory or pass the AppImage path:" >&2
    echo "  ./install.sh /path/to/$APPIMAGE_NAME" >&2
    exit 1
  fi

  local assets_dir
  assets_dir="$(find_assets_dir)"

  if [[ -z "$assets_dir" ]]; then
    echo "Error: could not find the assets directory (icon + .desktop file)." >&2
    exit 1
  fi

  echo "Installing Obsidian Terminal..."
  echo ""

  # -- binary ---------------------------------------------------------------
  mkdir -p "$BIN_DIR"
  cp "$appimage" "$BIN_DIR/$APP_NAME"
  chmod +x "$BIN_DIR/$APP_NAME"
  echo "  binary  → $BIN_DIR/$APP_NAME"

  # -- desktop entry --------------------------------------------------------
  mkdir -p "$APPS_DIR"
  sed "s|^Exec=.*|Exec=$BIN_DIR/$APP_NAME|" \
    "$assets_dir/$DESKTOP_ID.desktop" \
    > "$APPS_DIR/$DESKTOP_ID.desktop"
  echo "  desktop → $APPS_DIR/$DESKTOP_ID.desktop"

  # -- icon -----------------------------------------------------------------
  mkdir -p "$ICON_DIR"
  cp "$assets_dir/icons/hicolor/64x64/apps/$DESKTOP_ID.png" \
    "$ICON_DIR/$DESKTOP_ID.png"
  echo "  icon    → $ICON_DIR/$DESKTOP_ID.png"

  # -- desktop database -----------------------------------------------------
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
  fi

  # -- PATH hint ------------------------------------------------------------
  if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo ""
    echo "Note: $BIN_DIR is not in your PATH."
    echo "Add it by appending this line to your shell profile (~/.bashrc or ~/.zshrc):"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
  fi

  echo ""
  echo "Obsidian has been installed."
  echo "Launch it from the app menu or run: $APP_NAME"
  echo ""
  echo "On first launch, the setup wizard will guide you through initial configuration."
}

main "$@"
