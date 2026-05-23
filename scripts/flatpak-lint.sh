#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/flatpak/io.github.vaulttec_dev.CosmicContainers.json"

flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest "$MANIFEST"
