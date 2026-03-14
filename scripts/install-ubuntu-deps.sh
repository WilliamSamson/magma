#!/usr/bin/env bash

set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libgtk-4-dev \
  libvte-2.91-gtk4-dev \
  libwebkitgtk-6.0-dev
