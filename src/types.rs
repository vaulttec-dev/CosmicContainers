//! Shared domain types and constants for the panel applet.

/// Desktop entry / application ID (reverse domain notation).
pub const APP_ID: &str = "dev.linuxpop.Extension";

/// Official Docker mark (brand blue `#2496ED`), same as app icon in settings.
pub const DOCKER_ICON_SVG: &[u8] =
    include_bytes!("../resources/icons/hicolor/scalable/apps/dev.linuxpop.Extension.svg");

/// Label for containers not started by Docker Compose (like Docker Desktop).
pub const STANDALONE_GROUP: &str = "Standalone";

/// Auto-refresh interval for panel and popup lists.
pub const POLL_INTERVAL_SECS: u64 = 5;

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
}

/// One row from `docker ps` with Compose labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerSummary {
    pub name: String,
    pub state: String,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
}

impl ContainerSummary {
    pub fn is_running(&self) -> bool {
        self.state.starts_with("running")
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
