# linux-pop-extension

COSMIC panel applet for **Pop!_OS** (COSMIC desktop). Starter widget that shows a smiley on the panel and can be added from system settings.

## Requirements

- **Pop!_OS 24.04+** (or another distro with COSMIC panel)
- Rust 1.85 or newer (edition 2024)
- System libraries for building `libcosmic`:

```bash
sudo apt install build-essential pkg-config \
  libxkbcommon-dev libwayland-dev libdbus-1-dev \
  libpipewire-0.3-dev libpam0g-dev libssl-dev \
  libegl1-mesa-dev libinput-dev libpulse-dev
```

- `rustfmt` and `clippy`:

```bash
rustup component add rustfmt clippy
```

## Install

```bash
git clone <repository-url>
cd linux-pop-extension
./scripts/install-local.sh
```

This installs to `~/.local` (binary, `.desktop`, AppStream metainfo, icon).

### Add the widget

1. Restart the panel or log out and back in (if the widget list does not update).
2. Open **Settings → Desktop → Panel → Widgets → Add widget**.
3. Select **Linux Pop Extension** → **Add**.
4. The panel shows **😀**.

## Development

```bash
# Library tests only (no libcosmic / system GUI deps)
cargo test --lib --no-default-features

# Full applet build
cargo build --release

# Run on COSMIC (from a graphical session)
cargo run --release
```

Quality checks:

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --lib --no-default-features && cargo test --all-features
```

- Agent instructions: [AGENTS.md](AGENTS.md)
- Git hooks: `.githooks/pre-commit` (SnakeFlow)

## API

| Item | Description |
|------|-------------|
| `APP_ID` | Desktop entry ID: `dev.linuxpop.Extension` |
| `PANEL_EMOJI` | Panel label text (`😀`) |

## Contributing

1. Fork and branch from `master`.
2. Run `cargo fmt`, `cargo clippy --all-features`, and tests.
3. Update [CHANGELOG.md](CHANGELOG.md) for user-visible changes.
4. Open a pull request; CI must pass.

Report security issues privately — see [SECURITY.md](SECURITY.md).

## License

MIT License. See [LICENSE](LICENSE).
