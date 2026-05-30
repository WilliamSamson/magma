#!/usr/bin/env bash
#
# Magma Terminal — Build Release
#
# Builds the Debian package and stages a GitHub-ready Linux asset.
#
# Outputs:
#   dist/magma_<version>_amd64.deb
#   dist/magma-v<version>-linux-x86_64.deb
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
PACKAGE_FILE="$DIST_DIR/magma_${DEB_VERSION}_amd64.deb"
RELEASE_FILE="$DIST_DIR/magma-v${CARGO_VERSION}-linux-x86_64.deb"

verify_deb() {
  local artifact="$1"

  if [[ ! -f "$artifact" ]]; then
    echo "Error: .deb was not created at $artifact" >&2
    exit 1
  fi

  if ! dpkg-deb --info "$artifact" >/dev/null 2>&1; then
    echo "Error: expected a Debian package, but $artifact is not a valid .deb archive" >&2
    exit 1
  fi
}

echo "Building Magma $CARGO_VERSION release..."
echo ""

# ---------------------------------------------------------------------------
# Build .deb package
# ---------------------------------------------------------------------------
"$ROOT_DIR/scripts/build-deb.sh"

verify_deb "$PACKAGE_FILE"
cp "$PACKAGE_FILE" "$RELEASE_FILE"
verify_deb "$RELEASE_FILE"

echo ""
echo "============================================"
echo "  Release artifact ready"
echo "============================================"
echo ""
echo "Package:"
echo "  $PACKAGE_FILE"
echo ""
echo "GitHub release asset:"
echo "  $RELEASE_FILE"
echo ""
echo "Upload to GitHub with:"
echo "  gh release create v$CARGO_VERSION $RELEASE_FILE --title \"Magma v$CARGO_VERSION\" --notes \"Beta release\""
