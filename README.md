# Cosmic Containers (cosmic-containers)

COSMIC panel applet for **Pop!_OS**: manage Docker containers from the panel — running count at a glance, list in a popup, **Play** / **Stop** without keeping a terminal open on `docker ps`.

## Requirements

- **Pop!_OS 24.04+** (or another distro with COSMIC panel)
- **Docker** CLI (`docker` in `PATH`, daemon running, user in `docker` group or rootless Docker)
- `ss` from **iproute2** and `kill` from **procps** — both preinstalled on Pop!_OS; required by the **Ports** tab
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
cd cosmic-containers
./scripts/install-local.sh
```

This installs to `~/.local` (binary, `.desktop`, AppStream metainfo, icon).

### Flatpak (COSMIC Store / cosmic remote)

Distribution channel: [COSMIC Flatpak](https://github.com/pop-os/cosmic-flatpak) (Flathub does not accept panel applets).

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

- Tabs: **Containers**, **Images**, **Volumes**, **Ports**, **Settings**
- **Containers**: grouped by Compose project, **hide stopped** filter (hide icon), collapse groups, icon actions (play / stop / reload / delete), published ports and live CPU / RAM usage (`docker stats`)
- **Images** / **Volumes**: list with delete (`docker rmi` / `docker volume rm`)
- **Ports**: your dev servers (`ss -tulpn`), sorted by port, IPv4/IPv6 rows merged. Three kinds are listed: TCP sockets this user is listening on; the UDP sockets of those same processes (a media/QUIC server such as `livekit-server --dev` binds both, so its RTC port is not lost); and root-owned TCP above port 1024, which is a `sudo`-started server — shown for diagnosis, but without a kill button, since `ss` hides whose it is and the signal would be denied. Deliberately hidden: UDP-only chatter (mDNS, DNS stub, NTP), root-owned ports below 1024 (system services), and ports published by a running container — containers belong to the **Containers** tab. Per row: **Kill** sends `SIGTERM`, **Force** sends `SIGKILL`. **Clean** (broom icon) signals every entry in the list at once. Signals go to the listener's **process group**, not the bare PID: dev servers run under supervisors (`concurrently → pnpm → turbo → next`) that respawn a killed child and re-grab the port. A stack sharing one supervisor therefore goes down together, and gets a single signal rather than one per port
- **Clean** (broom icon, top right): prunes the active tab — stopped containers, unused images (`-a`), or unused volumes; asks for confirmation and reports reclaimed space
- **Settings**: panel running-count font size (10–28 pt), live preview; saved to `$XDG_CONFIG_HOME/io.github.vaulttec_dev.CosmicContainers/settings.conf` (migrated automatically from `~/.config/linux-pop-extension/` when present)
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
| `PortEntry` | Listening socket: port, protocol, bind addresses, owning process |
| `parse_ss_output` | Parse `ss -tulpn` stdout, merging IPv4/IPv6 rows per port |

## Contributing

1. Fork and branch from `master`.
2. Run `cargo fmt`, `cargo clippy --all-features`, and tests.
3. Update [CHANGELOG.md](CHANGELOG.md) for user-visible changes.
4. Open a pull request; CI must pass.

Report security issues privately — see [SECURITY.md](SECURITY.md).

## License

MIT License. See [LICENSE](LICENSE).

Docker is a trademark of Docker, Inc. This project is not affiliated with, endorsed by, or
sponsored by Docker, Inc.; it only drives the `docker` command-line client and does not ship
any Docker branding. The application icon and the bundled action glyphs were drawn for this
project and are covered by the project license; the references in `src/action_icons.rs` name
the designs they follow.
