#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
NAME="linux-pop-extension"
APP_ID="io.github.vaulttec_dev.CosmicContainers"

echo "Building release binary..."
cargo build --release --manifest-path "$ROOT/Cargo.toml"

BIN_SRC="$ROOT/target/release/$NAME"
BIN_DST="$PREFIX/bin/$NAME"
DESKTOP_DST="$PREFIX/share/applications/$APP_ID.desktop"
METAINFO_DST="$PREFIX/share/metainfo/$APP_ID.metainfo.xml"
ICON_DST="$PREFIX/share/icons/hicolor/scalable/apps/$APP_ID.svg"

install -Dm0755 "$BIN_SRC" "$BIN_DST"
install -Dm0644 "$ROOT/resources/$APP_ID.desktop" "$DESKTOP_DST"
install -Dm0644 "$ROOT/resources/$APP_ID.metainfo.xml" "$METAINFO_DST"
install -Dm0644 "$ROOT/resources/icons/hicolor/scalable/apps/$APP_ID.svg" "$ICON_DST"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$PREFIX/share/applications" || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" || true
fi

echo "Installed to $PREFIX"
echo "Add widget: Settings → Desktop → Panel → Widgets → Add widget → Cosmic Containers"
