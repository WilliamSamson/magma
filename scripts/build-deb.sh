#!/usr/bin/env bash
#
# Obsidian Terminal — Build .deb Package
#
# Produces a Debian package that can be installed with:
#   sudo dpkg -i obsidian_<version>_amd64.deb
#
# If runtime dependencies are missing:
#   sudo apt-get install -f
#

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
CARGO_TOML="$ROOT_DIR/Cargo.toml"
DESKTOP_ID="io.obsidian.terminal"

# ---------------------------------------------------------------------------
# Read metadata from Cargo.toml
# ---------------------------------------------------------------------------
CARGO_VERSION="$(grep -m1 '^version' "$CARGO_TOML" | sed 's/.*"\(.*\)".*/\1/')"
# Debian uses ~ for pre-release: 0.1.0-beta.1 → 0.1.0~beta.1
VERSION="$(echo "$CARGO_VERSION" | sed 's/-/~/')"
ARCH="amd64"
PKG_NAME="obsidian"
DEB_NAME="${PKG_NAME}_${VERSION}_${ARCH}"
DEB_ROOT="$DIST_DIR/$DEB_NAME"

echo "Building Obsidian $VERSION .deb package..."
echo ""

# ---------------------------------------------------------------------------
# Build release binary
# ---------------------------------------------------------------------------
cargo build --release --manifest-path "$CARGO_TOML"

BIN_PATH="$ROOT_DIR/target/release/obsidian"
if [[ ! -f "$BIN_PATH" ]]; then
  echo "Error: release binary not found at $BIN_PATH" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Assemble the .deb directory tree
# ---------------------------------------------------------------------------
rm -rf "$DEB_ROOT"
mkdir -p \
  "$DEB_ROOT/DEBIAN" \
  "$DEB_ROOT/usr/bin" \
  "$DEB_ROOT/usr/share/applications" \
  "$DEB_ROOT/usr/share/icons/hicolor/64x64/apps"

# -- control file -----------------------------------------------------------
INSTALLED_SIZE=$(du -sk "$BIN_PATH" | awk '{print $1}')

cat > "$DEB_ROOT/DEBIAN/control" <<EOF
Package: $PKG_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Installed-Size: $INSTALLED_SIZE
Depends: libgtk-4-1, libvte-2.91-gtk4-0, libwebkitgtk-6.0-4
Maintainer: Obsidian Terminal <obsidian@localhost>
Description: GPU-accelerated terminal emulator
 A modern GTK4 terminal workspace with an embedded log viewer,
 web browser pane, and a first-run setup wizard.
 Built with Rust, VTE4, and WebKitGTK.
Homepage: https://github.com/WilliamSamson/obsidian
EOF

# -- binary -----------------------------------------------------------------
cp "$BIN_PATH" "$DEB_ROOT/usr/bin/$PKG_NAME"
chmod 755 "$DEB_ROOT/usr/bin/$PKG_NAME"
strip "$DEB_ROOT/usr/bin/$PKG_NAME" 2>/dev/null || true

# -- desktop entry ----------------------------------------------------------
cp "$ROOT_DIR/assets/$DESKTOP_ID.desktop" \
  "$DEB_ROOT/usr/share/applications/$DESKTOP_ID.desktop"

# -- icon -------------------------------------------------------------------
cp "$ROOT_DIR/assets/icons/hicolor/64x64/apps/$DESKTOP_ID.png" \
  "$DEB_ROOT/usr/share/icons/hicolor/64x64/apps/$DESKTOP_ID.png"

# -- postinst: refresh desktop caches after install -------------------------
cat > "$DEB_ROOT/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
EOF
chmod 755 "$DEB_ROOT/DEBIAN/postinst"

# -- postrm: refresh caches after removal ----------------------------------
cat > "$DEB_ROOT/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
EOF
chmod 755 "$DEB_ROOT/DEBIAN/postrm"

# ---------------------------------------------------------------------------
# Build the .deb
# ---------------------------------------------------------------------------
mkdir -p "$DIST_DIR"
dpkg-deb --build --root-owner-group "$DEB_ROOT" "$DIST_DIR/${DEB_NAME}.deb"

echo ""
echo ".deb package created:"
echo "  $DIST_DIR/${DEB_NAME}.deb"
echo ""
echo "Install with:"
echo "  sudo dpkg -i $DIST_DIR/${DEB_NAME}.deb"
echo "  sudo apt-get install -f   # if dependencies are missing"
