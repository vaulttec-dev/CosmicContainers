# Cosmic Containers (linux-pop-extension)

COSMIC panel applet for **Pop!_OS**: manage Docker containers from the panel — running count at a glance, list in a popup, **Play** / **Stop** without keeping a terminal open on `docker ps`.

## Requirements

- **Pop!_OS 24.04+** (or another distro with COSMIC panel)
- **Docker** CLI (`docker` in `PATH`, daemon running, user in `docker` group or rootless Docker)
- Rust 1.93 or newer (edition 2024; required by `libcosmic`)
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

### Flatpak (COSMIC Store / cosmic remote)

Distribution channel: [COSMIC Flatpak](https://github.com/pop-os/cosmic-flatpak) (Flathub does not accept panel applets). Track submission: [pop-os/cosmic-flatpak#178](https://github.com/pop-os/cosmic-flatpak/pull/178).

Once merged and published:

```bash
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install cosmic io.github.vaulttec_dev.CosmicContainers
```

Requires **Docker on the host** (daemon running, user in `docker` group or rootless Docker). The Flatpak uses `flatpak-spawn --host docker` to reach the host CLI.

Build locally from this repository:

```bash
sudo apt install flatpak flatpak-builder
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install -y flathub org.flatpak.Builder com.system76.Cosmic.BaseApp//stable \
  org.freedesktop.Platform//25.08 org.freedesktop.Sdk//25.08 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08
flatpak-builder --user --install --force-clean build-dir \
  flatpak/io.github.vaulttec_dev.CosmicContainers.json
```

### Add the widget

1. Restart the panel or log out and back in (if the widget list does not update).
2. Open **Settings → Desktop → Panel → Widgets → Add widget**.
3. Select **Cosmic Containers** (desktop id: `io.github.vaulttec_dev.CosmicContainers`) → **Add**.
4. The panel shows the number of **running** containers (`0`, `1`, …). Click to open the list.

## Usage

| Panel | Meaning |
|-------|---------|
| `N` | `N` containers in `running` state |
| `!` | Docker error (not installed, daemon down, or permission denied) |

In the popup:

- Tabs: **Containers**, **Images**, **Volumes**, **Settings**
- **Containers**: grouped by Compose project, **hide stopped** filter (hide icon), collapse groups, icon actions (play / stop / reload / delete)
- **Images** / **Volumes**: list with delete (`docker rmi` / `docker volume rm`) and refresh
- **Settings**: panel running-count font size (10–28 pt), live preview; saved to `~/.config/io.github.vaulttec_dev.CosmicContainers/settings.conf` (migrated automatically from `~/.config/linux-pop-extension/` when present)
- Auto-refresh every 5 seconds (panel count + open popup)

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
| `APP_ID` | Desktop entry ID: `io.github.vaulttec_dev.CosmicContainers` |
| `ContainerSummary` | Parsed `name` + `state` from `docker ps` |
| `parse_ps_output` | Parse CLI stdout (`name\|state` per line) |
| `running_count` | Count containers with `running` state |

## Contributing

1. Fork and branch from `master`.
2. Run `cargo fmt`, `cargo clippy --all-features`, and tests.
3. Update [CHANGELOG.md](CHANGELOG.md) for user-visible changes.
4. Open a pull request; CI must pass.

Report security issues privately — see [SECURITY.md](SECURITY.md).

## License

MIT License. See [LICENSE](LICENSE).
