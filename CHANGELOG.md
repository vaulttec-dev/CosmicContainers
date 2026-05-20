# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Cosmic Containers**: panel badge with running Docker container count
- Popup tabs: **Containers**, **Images**, **Volumes**
- Compose grouping with **service name**, collapse/expand, **Start/Stop/Restart all** per group
- SVG action icons (play / stop / reload / delete / hide filter) from [SVG Repo](https://www.svgrepo.com/)
- Icon actions per container; **remove** stopped containers (with confirm)
- **Images** / **Volumes**: delete with confirm (`docker rmi`, `docker volume rm`)
- **Hide stopped** filter (hide icon); auto-refresh every 5 seconds
- `src/services/images.rs`, `src/services/volumes.rs`, Docker CLI parsers and unit tests

### Changed

- Replaced smiley demo with Docker manager UI
- Desktop/metainfo branding: **Cosmic Containers**

### Added (initial scaffold)

- COSMIC panel applet using `libcosmic` with popup support
- Desktop entry and AppStream metainfo for **Add widget** in COSMIC settings
- `scripts/install-local.sh` for `~/.local` installation
- Optional `applet` feature so library tests build without GUI system libraries

## [0.1.0] - 2026-05-20

### Added

- Initial Rust crate layout (`lib` + `bin`, integration smoke test)
- SnakeFlow / pre-commit quality configuration
- README, LICENSE, SECURITY.md, CODEOWNERS, Dependabot, CI workflow
