#!/usr/bin/env bash
# Sequential Rust checks for pre-commit (avoids parallel cargo file locks).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo check --all-features --locked
cargo test --lib --no-default-features --locked
cargo test --all-features --locked
