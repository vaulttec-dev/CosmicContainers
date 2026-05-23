//! Shared domain types and constants for the panel applet.

/// Desktop entry / application ID (reverse domain notation).
pub const APP_ID: &str = "io.github.vaulttec_dev.CosmicContainers";

/// Official Docker mark (brand blue `#2496ED`), same as app icon in settings.
pub const DOCKER_ICON_SVG: &[u8] = include_bytes!(
    "../resources/icons/hicolor/scalable/apps/io.github.vaulttec_dev.CosmicContainers.svg"
);

/// Label for containers not started by Docker Compose (like Docker Desktop).
pub const STANDALONE_GROUP: &str = "Standalone";

/// Auto-refresh interval for panel and popup lists.
pub const POLL_INTERVAL_SECS: u64 = 5;

/// Maximum list rows shown per popup tab page.
pub const LIST_PAGE_SIZE: usize = 8;

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
    Settings,
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
