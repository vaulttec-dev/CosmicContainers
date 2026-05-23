#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/flatpak/local-test.json"
BUILD_DIR="$ROOT/build-dir"

flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install -y --user flathub org.flatpak.Builder \
  com.system76.Cosmic.BaseApp//stable \
  org.freedesktop.Platform//25.08 \
  org.freedesktop.Sdk//25.08 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08

flatpak run org.flatpak.Builder --user --install --force-clean "$BUILD_DIR" "$MANIFEST"

echo "Installed io.github.vaulttec_dev.CosmicContainers (local build)."
echo "Add widget: Settings → Desktop → Panel → Widgets → Cosmic Containers"
