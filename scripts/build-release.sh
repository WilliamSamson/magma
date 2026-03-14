#!/usr/bin/env bash
#
# Obsidian Terminal — Build Release
#
# Builds the .deb package for GitHub release.
#
# Output:  dist/obsidian_<version>_amd64.deb
#
# Usage:
#   ./scripts/build-release.sh
#

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
CARGO_TOML="$ROOT_DIR/Cargo.toml"

CARGO_VERSION="$(grep -m1 '^version' "$CARGO_TOML" | sed 's/.*"\(.*\)".*/\1/')"
DEB_VERSION="$(echo "$CARGO_VERSION" | sed 's/-/~/')"
DEB_FILE="$DIST_DIR/obsidian_${DEB_VERSION}_amd64.deb"

echo "Building Obsidian $CARGO_VERSION release..."
echo ""

# ---------------------------------------------------------------------------
# Build .deb package
# ---------------------------------------------------------------------------
"$ROOT_DIR/scripts/build-deb.sh"

if [[ ! -f "$DEB_FILE" ]]; then
  echo "Error: .deb was not created at $DEB_FILE" >&2
  exit 1
fi

echo ""
echo "============================================"
echo "  Release artifact ready"
echo "============================================"
echo ""
echo "  $DEB_FILE"
echo ""
echo "Upload to GitHub with:"
echo "  gh release create v$CARGO_VERSION $DEB_FILE --title \"Obsidian v$CARGO_VERSION\" --notes \"Beta release\""
