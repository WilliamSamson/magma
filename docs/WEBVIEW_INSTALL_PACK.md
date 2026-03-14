# Web Viewer Install Pack

This project uses GTK4, VTE, and WebKitGTK for the desktop shell and embedded web viewer.

## Ubuntu 24.04 packages

Install these system packages before building:

- `build-essential`
- `pkg-config`
- `libgtk-4-dev`
- `libvte-2.91-gtk4-dev`
- `libwebkitgtk-6.0-dev`

## One-command install

```bash
./scripts/install-ubuntu-deps.sh
```

## Manual install

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libgtk-4-dev \
  libvte-2.91-gtk4-dev \
  libwebkitgtk-6.0-dev
```

## Build check

```bash
cargo check
cargo build
```
