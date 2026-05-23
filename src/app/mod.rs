mod popup;
mod update;
mod view;

use std::collections::HashSet;
use std::time::Duration;

use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::iced::theme::Style;
use cosmic::iced::window;
use cosmic::iced::window::Id;
use linux_pop_extension::{
    ContainerGroup, ContainerSummary, ImageSummary, POLL_INTERVAL_SECS, PopupTab, UserSettings,
    VolumeSummary, group_containers, running_count, settings,
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

/// Per-tab zero-based page index for paginated list tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TabPages {
    pub(super) containers: usize,
    pub(super) images: usize,
    pub(super) volumes: usize,
}

impl TabPages {
    pub(super) fn page_mut(&mut self, tab: PopupTab) -> Option<&mut usize> {
        match tab {
            PopupTab::Containers => Some(&mut self.containers),
            PopupTab::Images => Some(&mut self.images),
            PopupTab::Volumes => Some(&mut self.volumes),
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
    pub(super) collapsed_groups: HashSet<String>,
    pub(super) tab_pages: TabPages,
    pub(super) pending_remove: Option<PendingRemove>,
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
    Refresh,
    TabSelected(PopupTab),
    TabPagePrev,
    TabPageNext,
    ToggleGroupCollapse(String),
    RequestRemove(PendingRemove),
    ConfirmRemove,
    CancelRemove,
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

    pub(super) fn clamp_tab_page(&mut self, tab: PopupTab) {
        use linux_pop_extension::{LIST_PAGE_SIZE, clamp_page, total_pages};

        let item_count = match tab {
            PopupTab::Containers => self.container_row_count(),
            PopupTab::Images => self.image_list.len(),
            PopupTab::Volumes => self.volume_list.len(),
            PopupTab::Settings => return,
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

    pub(super) fn fetch_tab_task(tab: PopupTab, silent: bool) -> Task<Message> {
        use linux_pop_extension::services::{docker, images, volumes};

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
            PopupTab::Settings => Task::none(),
        }
    }

    pub(super) fn needs_data_fetch(tab: PopupTab) -> bool {
        !matches!(tab, PopupTab::Settings)
    }

    pub(super) fn fetch_active_tab(&self, silent: bool) -> Task<Message> {
        Self::fetch_tab_task(self.active_tab, silent)
    }
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = linux_pop_extension::APP_ID;

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
