# linux-pop-extension

Early-stage Rust binary for a Linux desktop extension. A Flathub release is planned.

## Install

### Requirements

- Rust 1.85 or newer (edition 2024)
- `rustfmt` and `clippy`:

  ```bash
  rustup component add rustfmt clippy
  ```

### Build from source

```bash
git clone <repository-url>
cd linux-pop-extension
cargo build --release
```

The release binary is at `target/release/linux-pop-extension`.

## Usage

Run the application:

```bash
cargo run
# or
./target/release/linux-pop-extension
```

Run the test suite:

```bash
cargo test
```

Run quality checks locally (SnakeFlow / pre-commit):

```bash
cargo fmt --check
cargo clippy -- -D warnings
./.githooks/pre-commit
```

## API

This crate exposes a small library surface for reuse and testing:

| Item | Description |
|------|-------------|
| `greeting()` | Returns the default greeting string (`src/lib.rs`). |

The binary entry point is `src/main.rs`; it prints the greeting to stdout.

## Development

- Project agent instructions: [AGENTS.md](AGENTS.md)
- Editor / Quality Hub settings: [.vscode/settings.json](.vscode/settings.json)
- Git hooks: `.githooks/pre-commit` (SnakeFlow gate)

Suggested workflow:

1. `cargo check` for fast feedback
2. `cargo test` before committing
3. Commit via IDE or `git commit` (pre-commit runs automatically when `core.hooksPath` is `.githooks`)

## Contributing

1. Fork the repository and create a branch from `master`.
2. Keep changes focused; run `cargo fmt`, `cargo clippy`, and `cargo test`.
3. Update [CHANGELOG.md](CHANGELOG.md) for user-visible changes.
4. Open a pull request; CI must pass (see `.github/workflows/ci.yml`).

Report security issues privately — see [SECURITY.md](SECURITY.md).

## License

MIT License. See [LICENSE](LICENSE).
