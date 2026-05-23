#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo fmt --check
bash scripts/cargo-precommit.sh
cargo clippy --all-features --locked -- -D warnings
cargo test --lib --no-default-features
cargo test --all-features
