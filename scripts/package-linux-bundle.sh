#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPDIR="${1:-$ROOT_DIR/dist/AppDir}"
BIN_PATH="$ROOT_DIR/target/release/magma"
DESKTOP_ID="io.magma.terminal"
WEBKIT_DIR_NAME="webkitgtk-6.0"
ICON_SIZES=(48 64 128 256 512)

system_libdir() {
  local lib
  lib="$(ldconfig -p | awk '/libwebkitgtk-6\.0\.so/{print $NF; exit}')"
  if [[ -z "$lib" ]]; then
    echo "Could not resolve libwebkitgtk-6.0 from ldconfig" >&2
    exit 1
  fi
  dirname "$lib"
}

copy_linked_libs() {
  local source="$1"
  ldd "$source" \
    | awk '/=> \//{print $3} /^\//{print $1}' \
    | while read -r lib; do
        [[ -f "$lib" ]] || continue
        cp -L "$lib" "$APPDIR/usr/lib/"
      done
}

write_apprun() {
  cat >"$APPDIR/AppRun" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export APPDIR="$HERE"
export MAGMA_BUNDLED_SHELL="$HERE/usr/lib/magma/bin/bash"
export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="$HERE/usr/share${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
export GSETTINGS_SCHEMA_DIR="$HERE/usr/share/glib-2.0/schemas"
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1

if [[ -d "$HERE/usr/lib/gdk-pixbuf-2.0" ]]; then
  export GDK_PIXBUF_MODULEDIR="$HERE/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders"
  export GDK_PIXBUF_MODULE_FILE="$HERE/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
fi

exec "$HERE/usr/bin/magma" "$@"
SH

  chmod +x "$APPDIR/AppRun"
}

copy_runtime_dirs() {
  local libdir="$1"

  mkdir -p "$APPDIR/usr/lib/$WEBKIT_DIR_NAME"
  cp -a "$libdir/$WEBKIT_DIR_NAME/." "$APPDIR/usr/lib/$WEBKIT_DIR_NAME/"

  if [[ -d "$libdir/gdk-pixbuf-2.0" ]]; then
    mkdir -p "$APPDIR/usr/lib/gdk-pixbuf-2.0"
    cp -a "$libdir/gdk-pixbuf-2.0/." "$APPDIR/usr/lib/gdk-pixbuf-2.0/"
  fi

  if [[ -d /usr/share/glib-2.0/schemas ]]; then
    mkdir -p "$APPDIR/usr/share/glib-2.0"
    cp -a /usr/share/glib-2.0/schemas "$APPDIR/usr/share/glib-2.0/"
  fi
}

main() {
  local libdir
  libdir="$(system_libdir)"

  cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

  rm -rf "$APPDIR"
  mkdir -p \
    "$APPDIR/usr/bin" \
    "$APPDIR/usr/lib" \
    "$APPDIR/usr/lib/magma/bin" \
    "$APPDIR/usr/share/applications"

  local size
  for size in "${ICON_SIZES[@]}"; do
    mkdir -p "$APPDIR/usr/share/icons/hicolor/${size}x${size}/apps"
  done

  cp "$BIN_PATH" "$APPDIR/usr/bin/magma"
  cp /bin/bash "$APPDIR/usr/lib/magma/bin/bash"

  cp "$ROOT_DIR/assets/$DESKTOP_ID.desktop" "$APPDIR/usr/share/applications/$DESKTOP_ID.desktop"
  cp "$ROOT_DIR/assets/$DESKTOP_ID.desktop" "$APPDIR/$DESKTOP_ID.desktop"
  cp "$ROOT_DIR/assets/icons/hicolor/256x256/apps/$DESKTOP_ID.png" "$APPDIR/$DESKTOP_ID.png"

  for size in "${ICON_SIZES[@]}"; do
    cp "$ROOT_DIR/assets/icons/hicolor/${size}x${size}/apps/$DESKTOP_ID.png" \
      "$APPDIR/usr/share/icons/hicolor/${size}x${size}/apps/$DESKTOP_ID.png"
  done

  copy_linked_libs "$APPDIR/usr/bin/magma"
  copy_linked_libs "$APPDIR/usr/lib/magma/bin/bash"

  for helper in WebKitNetworkProcess WebKitWebProcess WebKitGPUProcess; do
    if [[ -x "$libdir/$WEBKIT_DIR_NAME/$helper" ]]; then
      copy_linked_libs "$libdir/$WEBKIT_DIR_NAME/$helper"
    fi
  done

  copy_runtime_dirs "$libdir"
  write_apprun

  echo "Linux bundle assembled at: $APPDIR"
}

main "$@"
