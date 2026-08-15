mod popup;
mod update;
mod view;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::iced::theme::Style;
use cosmic::iced::window;
use cosmic::iced::window::Id;
use cosmic_containers::{
    ContainerGroup, ContainerStats, ContainerSummary, ImageSummary, POLL_INTERVAL_SECS, PopupTab,
    PortEntry, UserSettings, VolumeSummary, group_containers, running_count, settings,
};

pub use popup::POPUP_HEIGHT;
pub use popup::POPUP_WIDTH;

/// Item awaiting delete confirmation in the popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingRemove {
    Container(String),
    Image(String),
    Volume(String),
}

/// Process awaiting kill confirmation in the ports tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingKill {
    pub pid: u32,
    pub label: String,
}

/// Per-tab zero-based page index for paginated list tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TabPages {
    pub(super) containers: usize,
    pub(super) images: usize,
    pub(super) volumes: usize,
    pub(super) ports: usize,
}

impl TabPages {
    pub(super) fn page_mut(&mut self, tab: PopupTab) -> Option<&mut usize> {
        match tab {
            PopupTab::Containers => Some(&mut self.containers),
            PopupTab::Images => Some(&mut self.images),
            PopupTab::Volumes => Some(&mut self.volumes),
            PopupTab::Ports => Some(&mut self.ports),
            PopupTab::Settings => None,
        }
    }
}

/// COSMIC panel applet state.
#[derive(Default)]
pub struct App {
    pub(super) core: Core,
    pub(super) popup: Option<Id>,
    pub(super) active_tab: PopupTab,
    pub(super) containers: Vec<ContainerSummary>,
    pub(super) groups: Vec<ContainerGroup>,
    pub(super) image_list: Vec<ImageSummary>,
    pub(super) volume_list: Vec<VolumeSummary>,
    pub(super) port_list: Vec<PortEntry>,
    pub(super) container_stats: HashMap<String, ContainerStats>,
    pub(super) collapsed_groups: HashSet<String>,
    pub(super) tab_pages: TabPages,
    pub(super) pending_remove: Option<PendingRemove>,
    pub(super) pending_kill: Option<PendingKill>,
    pub(super) pending_prune: bool,
    pub(super) notice: Option<String>,
    pub(super) loading: bool,
    pub(super) error: Option<String>,
    pub(super) settings: UserSettings,
}

/// Messages from the panel button, popup, and Docker CLI results.
#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    TogglePopup,
    Poll,
    TabSelected(PopupTab),
    TabPagePrev,
    TabPageNext,
    ToggleGroupCollapse(String),
    RequestRemove(PendingRemove),
    ConfirmRemove,
    CancelRemove,
    RequestKill(PendingKill),
    ConfirmKill {
        force: bool,
    },
    CancelKill,
    StopContainerPort(String),
    RequestPrune,
    ConfirmPrune,
    CancelPrune,
    PruneCompleted(Result<String, String>),
    StatsLoaded(Result<Vec<ContainerStats>, String>),
    ContainersLoaded {
        result: Result<Vec<ContainerSummary>, String>,
        silent: bool,
    },
    ImagesLoaded {
        result: Result<Vec<ImageSummary>, String>,
        silent: bool,
    },
    VolumesLoaded {
        result: Result<Vec<VolumeSummary>, String>,
        silent: bool,
    },
    PortsLoaded {
        result: Result<Vec<PortEntry>, String>,
        silent: bool,
    },
    Start(String),
    Stop(String),
    Restart(String),
    StartGroup(String),
    StopGroup(String),
    RestartGroup(String),
    PanelCountFontSizeChanged(u16),
}

impl App {
    pub(super) fn panel_label(&self) -> String {
        if self.error.is_some() && self.containers.is_empty() {
            return "!".to_string();
        }
        format!("{}", running_count(&self.containers))
    }

    pub(super) fn rebuild_groups(&mut self) {
        self.groups = group_containers(self.containers.clone());
    }

    pub(super) fn container_row_count(&self) -> usize {
        self.groups.iter().fold(0, |count, group| {
            let collapsed = self.collapsed_groups.contains(&group.project);
            count + 1 + if collapsed { 0 } else { group.containers.len() }
        })
    }

    /// Rows a paginated tab currently holds (`None` for tabs without a list).
    pub(super) fn tab_item_count(&self, tab: PopupTab) -> Option<usize> {
        match tab {
            PopupTab::Containers => Some(self.container_row_count()),
            PopupTab::Images => Some(self.image_list.len()),
            PopupTab::Volumes => Some(self.volume_list.len()),
            PopupTab::Ports => Some(self.port_list.len()),
            PopupTab::Settings => None,
        }
    }

    pub(super) fn clamp_tab_page(&mut self, tab: PopupTab) {
        use cosmic_containers::{LIST_PAGE_SIZE, clamp_page, total_pages};

        let Some(item_count) = self.tab_item_count(tab) else {
            return;
        };
        let total = total_pages(item_count, LIST_PAGE_SIZE);
        let Some(page) = self.tab_pages.page_mut(tab) else {
            return;
        };
        *page = clamp_page(*page, total);
    }

    pub(super) fn apply_containers(
        &mut self,
        result: Result<Vec<ContainerSummary>, String>,
        silent: bool,
    ) {
        if !silent {
            self.loading = false;
        }
        match result {
            Ok(containers) => {
                self.containers = containers;
                self.rebuild_groups();
                self.clamp_tab_page(PopupTab::Containers);
                if self.active_tab == PopupTab::Containers {
                    self.error = None;
                }
            }
            Err(err) => {
                if self.active_tab == PopupTab::Containers {
                    self.error = Some(err);
                }
            }
        }
    }

    pub(super) fn apply_images(&mut self, result: Result<Vec<ImageSummary>, String>, silent: bool) {
        if !silent {
            self.loading = false;
        }
        match result {
            Ok(image_list) => {
                self.image_list = image_list;
                self.clamp_tab_page(PopupTab::Images);
                if self.active_tab == PopupTab::Images {
                    self.error = None;
                }
            }
            Err(err) => {
                if self.active_tab == PopupTab::Images {
                    self.error = Some(err);
                }
            }
        }
    }

    pub(super) fn apply_volumes(
        &mut self,
        result: Result<Vec<VolumeSummary>, String>,
        silent: bool,
    ) {
        if !silent {
            self.loading = false;
        }
        match result {
            Ok(volume_list) => {
                self.volume_list = volume_list;
                self.clamp_tab_page(PopupTab::Volumes);
                if self.active_tab == PopupTab::Volumes {
                    self.error = None;
                }
            }
            Err(err) => {
                if self.active_tab == PopupTab::Volumes {
                    self.error = Some(err);
                }
            }
        }
    }

    pub(super) fn apply_ports(&mut self, result: Result<Vec<PortEntry>, String>, silent: bool) {
        if !silent {
            self.loading = false;
        }
        match result {
            Ok(port_list) => {
                self.port_list = port_list;
                self.clamp_tab_page(PopupTab::Ports);
                if self.active_tab == PopupTab::Ports {
                    self.error = None;
                }
            }
            Err(err) => {
                if self.active_tab == PopupTab::Ports {
                    self.error = Some(err);
                }
            }
        }
    }

    /// Running container publishing `port`, if any (`docker-proxy` hides the real owner).
    pub(super) fn container_for_port(&self, port: u16) -> Option<&str> {
        self.containers
            .iter()
            .filter(|container| container.is_running())
            .find(|container| container.publishes_host_port(port))
            .map(|container| container.name.as_str())
    }

    pub(super) fn apply_stats(&mut self, result: Result<Vec<ContainerStats>, String>) {
        // Resource usage is supplementary: a failing `docker stats` must not mask the list.
        self.container_stats = match result {
            Ok(stats) => stats
                .into_iter()
                .map(|entry| (entry.name.clone(), entry))
                .collect(),
            Err(_) => HashMap::new(),
        };
    }

    pub(super) fn stats_task() -> Task<Message> {
        use cosmic_containers::services::stats;

        Task::perform(stats::list_stats(), |result| {
            cosmic::Action::App(Message::StatsLoaded(result))
        })
    }

    pub(super) fn fetch_tab_task(tab: PopupTab, silent: bool) -> Task<Message> {
        use cosmic_containers::services::{docker, images, ports, volumes};

        match tab {
            PopupTab::Containers => Task::perform(docker::list_containers(), move |result| {
                cosmic::Action::App(Message::ContainersLoaded { result, silent })
            }),
            PopupTab::Images => Task::perform(images::list_images(), move |result| {
                cosmic::Action::App(Message::ImagesLoaded { result, silent })
            }),
            PopupTab::Volumes => Task::perform(volumes::list_volumes(), move |result| {
                cosmic::Action::App(Message::VolumesLoaded { result, silent })
            }),
            PopupTab::Ports => Task::perform(ports::list_ports(), move |result| {
                cosmic::Action::App(Message::PortsLoaded { result, silent })
            }),
            PopupTab::Settings => Task::none(),
        }
    }

    pub(super) fn needs_data_fetch(tab: PopupTab) -> bool {
        !matches!(tab, PopupTab::Settings)
    }

    /// Fetch the active tab's list, plus live stats when the containers tab is shown.
    pub(super) fn fetch_active_tab(&self, silent: bool) -> Task<Message> {
        let list = Self::fetch_tab_task(self.active_tab, silent);
        if self.active_tab == PopupTab::Containers {
            return Task::batch([list, Self::stats_task()]);
        }
        list
    }
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = cosmic_containers::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        (
            Self {
                core,
                settings: settings::load(),
                ..Default::default()
            },
            Self::fetch_tab_task(PopupTab::Containers, false),
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::time::every(Duration::from_secs(POLL_INTERVAL_SECS)).map(|_| Message::Poll)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        self.handle_message(message)
    }

    fn view(&self) -> cosmic::Element<'_, Self::Message> {
        view::panel_view(self)
    }

    fn view_window(&self, _id: Id) -> cosmic::Element<'_, Self::Message> {
        view::empty_window()
    }

    fn style(&self) -> Option<Style> {
        Some(cosmic::applet::style())
    }
}
