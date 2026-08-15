# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Ports** tab: listening TCP/UDP sockets on the host (`ss -tulpn`), sorted by port with IPv4/IPv6 rows merged, showing the owning process and PID
- Free a port from the popup: `docker stop` for ports published by a running container, `kill -TERM` (or `-KILL` via **Force**) for any other process, both behind a confirmation row

## [0.2.0] - 2026-07-25

### Added

- Flatpak packaging manifest (`flatpak/io.github.vaulttec_dev.CosmicContainers.json`) for COSMIC Flatpak / Flathub
- Host Docker access in Flatpak via `flatpak-spawn --host docker`
- Tab-scoped clean action (broom icon) running `docker container|image|volume prune`, with confirmation and reclaimed-space feedback
- Published ports and live CPU / RAM usage per container (`docker stats --no-stream`, polled only while the popup is open)

### Changed

- Application ID renamed to `io.github.vaulttec_dev.CosmicContainers` (desktop entry, metainfo, icon paths)
- Crate, binary, and command renamed from `linux-pop-extension` to `cosmic-containers`; settings from the old config directory are migrated on first load
- Application icon replaced with an original design — the previous one was the Docker logo, a trademark of Docker, Inc.
- Popup doubled in size (800×960) with 16 rows per page
- Header refresh button replaced by the clean action; lists still auto-refresh every 5 seconds
- Flatpak permissions trimmed: dropped `--filesystem=/var/run/docker.sock` (unused — the applet talks to the host CLI via `flatpak-spawn`) and both `xdg-config` overrides; added `--socket=fallback-x11`

### Fixed

- Settings are written to `$XDG_CONFIG_HOME` when set; inside Flatpak `$HOME/.config` is a sandbox tmpfs, so the panel font size was lost on every restart
- Panel button grows along the panel's major axis instead of being fixed to one icon width, and the applet surface is wrapped in `autosize_window`, so the running count is no longer clipped; the font is capped to the panel thickness
- Volume prune passes `-a`: without it Docker removes only anonymous volumes and leaves unused named ones behind
- Ports are also shown for stopped containers, read from their configured bindings (`docker ps` reports ports only while a container runs)
- Settings path follows `APP_ID`; legacy config under `~/.config/linux-pop-extension/` is migrated on first load

## [0.1.0] - 2026-05-20

### Added

- Initial Rust crate layout (`lib` + `bin`, integration smoke test)
- SnakeFlow / pre-commit quality configuration
- README, LICENSE, SECURITY.md, CODEOWNERS, Dependabot, CI workflow
