use cosmic::app::Task;
use cosmic::surface::action::{app_popup, destroy_popup};
use linux_pop_extension::PopupTab;
use linux_pop_extension::services::{docker, images, volumes};

use super::popup::popup_view;
use super::{App, Message, POPUP_HEIGHT, POPUP_WIDTH, PendingRemove};

impl App {
    pub(super) fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => self.on_popup_closed(id),
            Message::TogglePopup => self.toggle_popup(),
            Message::Poll => self.poll_lists(),
            Message::Refresh => self.refresh_active_tab(),
            Message::TabSelected(tab) => self.select_tab(tab),
            Message::TabPagePrev => self.tab_page_prev(),
            Message::TabPageNext => self.tab_page_next(),
            Message::ToggleGroupCollapse(project) => self.toggle_group_collapse(project),
            Message::RequestRemove(target) => self.request_remove(target),
            Message::CancelRemove => self.cancel_remove(),
            Message::ConfirmRemove => self.confirm_remove(),
            Message::ContainersLoaded { result, silent } => {
                self.apply_containers(result, silent);
                Task::none()
            }
            Message::ImagesLoaded { result, silent } => {
                self.apply_images(result, silent);
                Task::none()
            }
            Message::VolumesLoaded { result, silent } => {
                self.apply_volumes(result, silent);
                Task::none()
            }
            Message::Start(name) => self.run_container_action(docker::start_then_list(name)),
            Message::Stop(name) => self.run_container_action(docker::stop_then_list(name)),
            Message::Restart(name) => self.run_container_action(docker::restart_then_list(name)),
            Message::StartGroup(project) => self.start_group(project),
            Message::StopGroup(project) => self.stop_group(project),
            Message::RestartGroup(project) => self.restart_group(project),
            Message::PanelCountFontSizeChanged(size) => self.set_panel_count_font_size(size),
        }
    }

    fn on_popup_closed(&mut self, id: cosmic::iced::window::Id) -> Task<Message> {
        if self.popup.as_ref() == Some(&id) {
            self.popup = None;
            self.pending_remove = None;
        }
        Task::none()
    }

    fn toggle_popup(&mut self) -> Task<Message> {
        if let Some(id) = self.popup.take() {
            self.pending_remove = None;
            return cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
                destroy_popup(id),
            )));
        }
        self.loading = true;
        let fetch = self.fetch_active_tab(false);
        let open = cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
            app_popup::<App>(
                |state: &mut App| {
                    let new_id = cosmic::iced::window::Id::unique();
                    state.popup = Some(new_id);
                    state.core.applet.get_popup_settings(
                        state.core.main_window_id().unwrap(),
                        new_id,
                        Some((POPUP_WIDTH as u32, POPUP_HEIGHT as u32)),
                        None,
                        None,
                    )
                },
                Some(Box::new(popup_view)),
            ),
        )));
        Task::batch([open, fetch])
    }

    fn poll_lists(&self) -> Task<Message> {
        let silent = true;
        let mut tasks = vec![Self::fetch_tab_task(PopupTab::Containers, silent)];
        if self.popup.is_some()
            && self.active_tab != PopupTab::Containers
            && App::needs_data_fetch(self.active_tab)
        {
            tasks.push(Self::fetch_tab_task(self.active_tab, silent));
        }
        Task::batch(tasks)
    }

    fn refresh_active_tab(&mut self) -> Task<Message> {
        if !App::needs_data_fetch(self.active_tab) {
            return Task::none();
        }
        self.loading = true;
        self.fetch_active_tab(false)
    }

    fn select_tab(&mut self, tab: PopupTab) -> Task<Message> {
        if self.active_tab == tab {
            return Task::none();
        }
        self.active_tab = tab;
        self.error = None;
        self.pending_remove = None;
        if !App::needs_data_fetch(tab) {
            self.loading = false;
            return Task::none();
        }
        self.loading = true;
        self.fetch_active_tab(false)
    }

    fn tab_page_prev(&mut self) -> Task<Message> {
        use linux_pop_extension::{LIST_PAGE_SIZE, clamp_page, total_pages};

        let tab = self.active_tab;
        let item_count = match tab {
            PopupTab::Containers => self.container_row_count(),
            PopupTab::Images => self.image_list.len(),
            PopupTab::Volumes => self.volume_list.len(),
            PopupTab::Settings => return Task::none(),
        };
        let total = total_pages(item_count, LIST_PAGE_SIZE);
        let Some(page) = self.tab_pages.page_mut(tab) else {
            return Task::none();
        };
        if *page > 0 {
            *page -= 1;
        }
        *page = clamp_page(*page, total);
        Task::none()
    }

    fn tab_page_next(&mut self) -> Task<Message> {
        use linux_pop_extension::{LIST_PAGE_SIZE, total_pages};

        let tab = self.active_tab;
        let item_count = match tab {
            PopupTab::Containers => self.container_row_count(),
            PopupTab::Images => self.image_list.len(),
            PopupTab::Volumes => self.volume_list.len(),
            PopupTab::Settings => return Task::none(),
        };
        let total = total_pages(item_count, LIST_PAGE_SIZE);
        let Some(page) = self.tab_pages.page_mut(tab) else {
            return Task::none();
        };
        if *page + 1 < total {
            *page += 1;
        }
        Task::none()
    }

    fn set_panel_count_font_size(&mut self, size: u16) -> Task<Message> {
        self.settings.set_panel_count_font_size(size);
        if let Err(err) = linux_pop_extension::settings::save(&self.settings) {
            self.error = Some(err);
        } else if self.active_tab == PopupTab::Settings {
            self.error = None;
        }
        Task::none()
    }

    fn toggle_group_collapse(&mut self, project: String) -> Task<Message> {
        if self.collapsed_groups.contains(&project) {
            self.collapsed_groups.remove(&project);
        } else {
            self.collapsed_groups.insert(project);
        }
        self.clamp_tab_page(PopupTab::Containers);
        Task::none()
    }

    fn request_remove(&mut self, target: PendingRemove) -> Task<Message> {
        self.pending_remove = Some(target);
        Task::none()
    }

    fn cancel_remove(&mut self) -> Task<Message> {
        self.pending_remove = None;
        Task::none()
    }

    fn confirm_remove(&mut self) -> Task<Message> {
        let Some(target) = self.pending_remove.take() else {
            return Task::none();
        };
        self.loading = true;
        match target {
            PendingRemove::Container(name) => {
                Task::perform(docker::remove_then_list(name), containers_loaded)
            }
            PendingRemove::Image(id) => Task::perform(images::remove_then_list(id), images_loaded),
            PendingRemove::Volume(name) => {
                Task::perform(volumes::remove_then_list(name), volumes_loaded)
            }
        }
    }

    fn run_container_action(
        &mut self,
        action: impl std::future::Future<
            Output = Result<Vec<linux_pop_extension::ContainerSummary>, String>,
        > + Send
        + 'static,
    ) -> Task<Message> {
        self.loading = true;
        Task::perform(action, containers_loaded)
    }

    fn start_group(&mut self, project: String) -> Task<Message> {
        let Some(group) = self.groups.iter().find(|g| g.project == project).cloned() else {
            self.loading = false;
            return Task::none();
        };
        self.loading = true;
        Task::perform(docker::start_group_then_list(group), containers_loaded)
    }

    fn stop_group(&mut self, project: String) -> Task<Message> {
        let Some(group) = self.groups.iter().find(|g| g.project == project).cloned() else {
            self.loading = false;
            return Task::none();
        };
        self.loading = true;
        Task::perform(docker::stop_group_then_list(group), containers_loaded)
    }

    fn restart_group(&mut self, project: String) -> Task<Message> {
        let Some(group) = self.groups.iter().find(|g| g.project == project).cloned() else {
            self.loading = false;
            return Task::none();
        };
        self.loading = true;
        Task::perform(docker::restart_group_then_list(group), containers_loaded)
    }
}

fn containers_loaded(
    result: Result<Vec<linux_pop_extension::ContainerSummary>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::ContainersLoaded {
        result,
        silent: false,
    })
}

fn images_loaded(
    result: Result<Vec<linux_pop_extension::ImageSummary>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::ImagesLoaded {
        result,
        silent: false,
    })
}

fn volumes_loaded(
    result: Result<Vec<linux_pop_extension::VolumeSummary>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::VolumesLoaded {
        result,
        silent: false,
    })
}
