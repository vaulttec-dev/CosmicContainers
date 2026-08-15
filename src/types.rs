//! Shared domain types and constants for the panel applet.

/// Desktop entry / application ID (reverse domain notation).
pub const APP_ID: &str = "io.github.vaulttec_dev.CosmicContainers";

/// Official Docker mark (brand blue `#2496ED`), same as app icon in settings.
pub const APP_ICON_SVG: &[u8] = include_bytes!(
    "../resources/icons/hicolor/scalable/apps/io.github.vaulttec_dev.CosmicContainers.svg"
);

/// Label for containers not started by Docker Compose (like Docker Desktop).
pub const STANDALONE_GROUP: &str = "Standalone";

/// Auto-refresh interval for panel and popup lists.
pub const POLL_INTERVAL_SECS: u64 = 5;

/// Maximum list rows shown per popup tab page.
pub const LIST_PAGE_SIZE: usize = 16;

/// Total pages for a list of `item_count` rows at `page_size` per page (minimum 1).
pub fn total_pages(item_count: usize, page_size: usize) -> usize {
    if page_size == 0 {
        return 1;
    }
    item_count.div_ceil(page_size).max(1)
}

/// Clamp a zero-based page index to `[0, total_pages - 1]`.
pub fn clamp_page(page: usize, total: usize) -> usize {
    page.min(total.saturating_sub(1))
}

/// Slice range for the given zero-based page.
pub fn page_range(page: usize, page_size: usize, len: usize) -> std::ops::Range<usize> {
    let start = page * page_size;
    let end = (start + page_size).min(len);
    start..end
}

/// Pop/COSMIC symbolic icon names (group collapse).
pub mod icons {
    pub const EXPAND: &str = "pan-end-symbolic";
    pub const COLLAPSE: &str = "pan-down-symbolic";
}

/// Active popup tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupTab {
    #[default]
    Containers,
    Images,
    Volumes,
    Ports,
    Settings,
}

impl PopupTab {
    /// Confirmation prompt for the tab-scoped prune action (`None` when unsupported).
    pub fn prune_prompt(self) -> Option<&'static str> {
        match self {
            PopupTab::Containers => Some("Remove all stopped containers?"),
            PopupTab::Images => Some("Remove all unused images?"),
            PopupTab::Volumes => Some("Remove all unused volumes?"),
            PopupTab::Ports | PopupTab::Settings => None,
        }
    }
}

/// One row from `docker ps` with Compose labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerSummary {
    pub name: String,
    pub state: String,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
    pub ports: Option<String>,
}

impl ContainerSummary {
    pub fn is_running(&self) -> bool {
        self.state.starts_with("running")
    }

    /// Compact published-port list (`8080→80, 5432`), IPv4/IPv6 duplicates merged.
    pub fn port_summary(&self) -> Option<String> {
        let raw = self.ports.as_deref()?.trim();
        if raw.is_empty() {
            return None;
        }
        let mut mappings: Vec<String> = Vec::new();
        for part in raw.split(',') {
            let mapping = format_port_mapping(part.trim());
            if !mapping.is_empty() && !mappings.contains(&mapping) {
                mappings.push(mapping);
            }
        }
        if mappings.is_empty() {
            None
        } else {
            Some(mappings.join(", "))
        }
    }

    /// True when this container publishes `port` on the host (`0.0.0.0:8080->80/tcp` → `8080`).
    pub fn publishes_host_port(&self, port: u16) -> bool {
        let Some(raw) = self.ports.as_deref() else {
            return false;
        };
        raw.split(',')
            .any(|part| mapping_covers_host_port(part.trim(), port))
    }

    pub fn group_label(&self) -> &str {
        self.compose_project
            .as_deref()
            .filter(|p| !p.is_empty())
            .unwrap_or(STANDALONE_GROUP)
    }

    /// Primary label for the list (service name when available).
    pub fn display_label(&self) -> String {
        match self.compose_service.as_deref() {
            Some(service) if !service.is_empty() => format!("{service} ({})", self.name),
            _ => self.name.clone(),
        }
    }
}

/// True when the host side of one mapping covers `port`.
///
/// Docker publishes ranges as a single entry (`0.0.0.0:9000-9001->9000-9001/tcp`),
/// so both endpoints have to be considered, not just a plain port number.
fn mapping_covers_host_port(part: &str, port: u16) -> bool {
    let Some((host, _)) = part.split_once("->") else {
        return false;
    };
    let host = host.rsplit(':').next().unwrap_or(host).trim();
    match host.split_once('-') {
        Some((first, last)) => match (first.parse::<u16>(), last.parse::<u16>()) {
            (Ok(first), Ok(last)) => (first..=last).contains(&port),
            _ => false,
        },
        None => host.parse() == Ok(port),
    }
}

/// `0.0.0.0:8080->80/tcp` → `8080→80`; `80/tcp` → `80`.
fn format_port_mapping(part: &str) -> String {
    match part.split_once("->") {
        Some((host, container)) => {
            let host_port = host.rsplit(':').next().unwrap_or(host).trim();
            let container_port = container.split('/').next().unwrap_or(container).trim();
            if host_port == container_port {
                container_port.to_string()
            } else {
                format!("{host_port}→{container_port}")
            }
        }
        None => part.split('/').next().unwrap_or(part).trim().to_string(),
    }
}

/// Containers grouped by Compose project (standalone last).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerGroup {
    pub project: String,
    pub containers: Vec<ContainerSummary>,
}

impl ContainerGroup {
    pub fn running_count(&self) -> usize {
        crate::grouping::running_count(&self.containers)
    }

    pub fn has_stopped(&self) -> bool {
        self.containers.iter().any(|c| !c.is_running())
    }

    pub fn has_running(&self) -> bool {
        self.running_count() > 0
    }
}

/// One row from `docker images --format`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageSummary {
    pub reference: String,
    pub size: String,
    pub id: String,
}

/// One row from `docker volume ls --format`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
}

/// One row from `docker stats --no-stream --format` (live resource usage).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerStats {
    pub name: String,
    pub cpu: String,
    pub memory: String,
}

impl ContainerStats {
    /// Single-line resource label for the container row.
    pub fn usage_line(&self) -> String {
        format!("CPU {} · RAM {}", self.cpu, self.memory)
    }
}

/// One listening socket from `ss -tulpn`, IPv4/IPv6 rows of the same port merged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortEntry {
    pub port: u16,
    /// `tcp` or `udp`, as reported by `ss`.
    pub protocol: String,
    /// Bind addresses seen for this port (`0.0.0.0`, `[::]`, `127.0.0.1`).
    pub addresses: Vec<String>,
    /// Process name; absent when the socket belongs to another user (root).
    pub process: Option<String>,
    pub pid: Option<u32>,
}

impl PortEntry {
    pub fn title(&self) -> String {
        format!("{}/{}", self.port, self.protocol)
    }

    /// Who holds the port: a container, a killable process, or an unreadable owner.
    pub fn owner_line(&self, container: Option<&str>) -> String {
        if let Some(container) = container {
            return format!("Docker · {container}");
        }
        match (&self.process, self.pid) {
            (Some(process), Some(pid)) => format!("{process} · PID {pid}"),
            (None, Some(pid)) => format!("PID {pid}"),
            _ => "PID hidden (owned by another user)".to_string(),
        }
    }

    pub fn address_line(&self) -> String {
        self.addresses.join(", ")
    }

    /// Label used in the kill confirmation prompt.
    pub fn kill_label(&self) -> String {
        match (&self.process, self.pid) {
            (Some(process), Some(pid)) => format!("{process} (PID {pid}) on {}", self.title()),
            (_, Some(pid)) => format!("PID {pid} on {}", self.title()),
            _ => self.title(),
        }
    }
}

#[cfg(test)]
mod pagination_tests {
    use super::{LIST_PAGE_SIZE, clamp_page, page_range, total_pages};

    #[test]
    fn total_pages_empty_list_is_one() {
        assert_eq!(total_pages(0, LIST_PAGE_SIZE), 1);
    }

    #[test]
    fn total_pages_exact_and_partial() {
        assert_eq!(total_pages(8, 8), 1);
        assert_eq!(total_pages(9, 8), 2);
    }

    #[test]
    fn clamp_page_stays_in_bounds() {
        assert_eq!(clamp_page(0, 1), 0);
        assert_eq!(clamp_page(5, 3), 2);
    }

    #[test]
    fn page_range_slices_correctly() {
        assert_eq!(page_range(0, 8, 5), 0..5);
        assert_eq!(page_range(1, 8, 10), 8..10);
    }
}
