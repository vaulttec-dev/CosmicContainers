# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Click popup on panel smiley (toggle open/close with greeting text)
- COSMIC panel applet using `libcosmic` (smiley `😀` on the panel)
- Desktop entry and AppStream metainfo for **Add widget** in COSMIC settings
- `scripts/install-local.sh` for `~/.local` installation
- Optional `applet` feature so library tests build without GUI system libraries

## [0.1.0] - 2026-05-20

### Added

- Initial Rust crate layout (`lib` + `bin`, integration smoke test)
- SnakeFlow / pre-commit quality configuration
- README, LICENSE, SECURITY.md, CODEOWNERS, Dependabot, CI workflow
