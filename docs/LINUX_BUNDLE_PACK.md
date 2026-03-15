# Linux Bundle Pack

This project can be packaged as a self-contained Linux bundle with:

- the `magma` binary
- a bundled `bash` shell
- GTK4, VTE4, and WebKitGTK shared libraries
- WebKit helper processes
- desktop and icon metadata

## Required host packages for building the bundle

- `build-essential`
- `pkg-config`
- `libgtk-4-dev`
- `libvte-2.91-gtk4-dev`
- `libwebkitgtk-6.0-dev`
- `glib2.0-bin`

## Bundle command

```bash
./scripts/package-linux-bundle.sh
```

Optional output path:

```bash
./scripts/package-linux-bundle.sh dist/Magma.AppDir
```

## Final AppImage

If `appimagetool` is installed:

```bash
./scripts/build-appimage.sh
```

Optional paths:

```bash
./scripts/build-appimage.sh dist/Magma.AppDir dist
```

## Bundle layout

The script creates an AppDir-style structure containing:

- `AppRun`
- `usr/bin/magma`
- `usr/lib/magma/bin/bash`
- `usr/lib/*.so*`
- `usr/lib/webkitgtk-6.0/*`
- `usr/share/applications/io.magma.terminal.desktop`
- `usr/share/icons/hicolor/64x64/apps/io.magma.terminal.png`

## Runtime behavior

At launch, `AppRun` sets:

- `MAGMA_BUNDLED_SHELL`
- `LD_LIBRARY_PATH`
- `XDG_DATA_DIRS`
- `GSETTINGS_SCHEMA_DIR`
- `WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS`

Magma then prefers the bundled shell before falling back to host shells.

## Notes

- The bundle script targets Linux desktop distribution.
- It is designed for AppDir/AppImage-style packaging.
- If you want a final `.AppImage`, run an AppImage tool against the generated AppDir after assembly.
