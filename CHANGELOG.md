# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Flatpak packaging manifest (`flatpak/io.github.vaulttec_dev.CosmicContainers.json`) for COSMIC Flatpak / Flathub
- Host Docker access in Flatpak via `flatpak-spawn --host docker`

### Changed

- Application ID renamed to `io.github.vaulttec_dev.CosmicContainers` (desktop entry, metainfo, icon paths)
- Settings path follows `APP_ID`; legacy config under `~/.config/linux-pop-extension/` is migrated on first load

## [0.1.0] - 2026-05-20

### Added

- Initial Rust crate layout (`lib` + `bin`, integration smoke test)
- SnakeFlow / pre-commit quality configuration
- README, LICENSE, SECURITY.md, CODEOWNERS, Dependabot, CI workflow
