#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPDIR="${1:-$ROOT_DIR/dist/AppDir}"
OUTPUT_DIR="${2:-$ROOT_DIR/dist}"

if ! command -v appimagetool >/dev/null 2>&1; then
  echo "appimagetool is required to build a final AppImage." >&2
  echo "Install appimagetool, then rerun this script." >&2
  exit 1
fi

"$ROOT_DIR/scripts/package-linux-bundle.sh" "$APPDIR"
mkdir -p "$OUTPUT_DIR"

APPIMAGE_OUTPUT="$OUTPUT_DIR/Magma-x86_64.AppImage"
ARCH=x86_64 appimagetool "$APPDIR" "$APPIMAGE_OUTPUT"

echo "Final AppImage created at: $APPIMAGE_OUTPUT"
