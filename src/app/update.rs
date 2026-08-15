use cosmic::app::Task;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic_containers::PopupTab;
use cosmic_containers::services::{docker, images, ports, volumes};

use super::popup::popup_view;
use super::{App, Message, POPUP_HEIGHT, POPUP_WIDTH, PendingKill, PendingRemove};

impl App {
    pub(super) fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => self.on_popup_closed(id),
            Message::TogglePopup => self.toggle_popup(),
            Message::Poll => self.poll_lists(),
            Message::TabSelected(tab) => self.select_tab(tab),
            Message::TabPagePrev => self.tab_page_prev(),
            Message::TabPageNext => self.tab_page_next(),
            Message::ToggleGroupCollapse(project) => self.toggle_group_collapse(project),
            Message::RequestRemove(target) => self.request_remove(target),
            Message::CancelRemove => self.cancel_remove(),
            Message::ConfirmRemove => self.confirm_remove(),
            Message::RequestKill(target) => self.request_kill(target),
            Message::CancelKill => self.cancel_kill(),
            Message::ConfirmKill { force } => self.confirm_kill(force),
            Message::StopContainerPort(name) => self.stop_container_port(name),
            Message::RequestPrune => self.request_prune(),
            Message::CancelPrune => self.cancel_prune(),
            Message::ConfirmPrune => self.confirm_prune(),
            Message::PruneCompleted(result) => self.prune_completed(result),
            Message::StatsLoaded(result) => {
                self.apply_stats(result);
                Task::none()
            }
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
            Message::PortsLoaded { result, silent } => {
                self.apply_ports(result, silent);
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
            self.pending_kill = None;
            self.pending_prune = false;
            self.notice = None;
        }
        Task::none()
    }

    fn toggle_popup(&mut self) -> Task<Message> {
        if let Some(id) = self.popup.take() {
            self.pending_remove = None;
            self.pending_kill = None;
            self.pending_prune = false;
            self.notice = None;
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
        if self.popup.is_some() {
            if self.active_tab == PopupTab::Containers {
                // `docker stats` is comparatively slow; only poll it with the popup open.
                tasks.push(Self::stats_task());
            } else if App::needs_data_fetch(self.active_tab) {
                tasks.push(Self::fetch_tab_task(self.active_tab, silent));
            }
        }
        Task::batch(tasks)
    }

    fn select_tab(&mut self, tab: PopupTab) -> Task<Message> {
        if self.active_tab == tab {
            return Task::none();
        }
        self.active_tab = tab;
        self.error = None;
        self.notice = None;
        self.pending_remove = None;
        self.pending_kill = None;
        self.pending_prune = false;
        if !App::needs_data_fetch(tab) {
            self.loading = false;
            return Task::none();
        }
        self.loading = true;
        self.fetch_active_tab(false)
    }

    fn tab_page_prev(&mut self) -> Task<Message> {
        use cosmic_containers::{LIST_PAGE_SIZE, clamp_page, total_pages};

        let tab = self.active_tab;
        let Some(item_count) = self.tab_item_count(tab) else {
            return Task::none();
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
        use cosmic_containers::{LIST_PAGE_SIZE, total_pages};

        let tab = self.active_tab;
        let Some(item_count) = self.tab_item_count(tab) else {
            return Task::none();
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
        if let Err(err) = cosmic_containers::settings::save(&self.settings) {
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
        self.pending_prune = false;
        self.pending_kill = None;
        self.pending_remove = Some(target);
        Task::none()
    }

    fn request_kill(&mut self, target: PendingKill) -> Task<Message> {
        self.pending_prune = false;
        self.pending_remove = None;
        self.pending_kill = Some(target);
        Task::none()
    }

    fn cancel_kill(&mut self) -> Task<Message> {
        self.pending_kill = None;
        Task::none()
    }

    /// Signal the confirmed process, then reload the port list.
    fn confirm_kill(&mut self, force: bool) -> Task<Message> {
        let Some(target) = self.pending_kill.take() else {
            return Task::none();
        };
        self.loading = true;
        self.notice = None;
        Task::perform(ports::kill_then_list(target.pid, force), ports_loaded)
    }

    /// Free a published port by stopping its container instead of killing `docker-proxy`.
    fn stop_container_port(&mut self, name: String) -> Task<Message> {
        self.loading = true;
        self.pending_kill = None;
        Task::perform(ports::stop_container_then_list(name), ports_loaded)
    }

    fn request_prune(&mut self) -> Task<Message> {
        if self.active_tab.prune_prompt().is_none() {
            return Task::none();
        }
        self.pending_remove = None;
        self.pending_kill = None;
        self.notice = None;
        self.pending_prune = true;
        Task::none()
    }

    fn cancel_prune(&mut self) -> Task<Message> {
        self.pending_prune = false;
        Task::none()
    }

    fn confirm_prune(&mut self) -> Task<Message> {
        self.pending_prune = false;
        let prune = match self.active_tab {
            PopupTab::Containers => Task::perform(docker::prune_containers(), prune_completed),
            PopupTab::Images => Task::perform(images::prune_images(), prune_completed),
            PopupTab::Volumes => Task::perform(volumes::prune_volumes(), prune_completed),
            PopupTab::Ports | PopupTab::Settings => return Task::none(),
        };
        self.loading = true;
        prune
    }

    fn prune_completed(&mut self, result: Result<String, String>) -> Task<Message> {
        match result {
            Ok(summary) => {
                self.notice = Some(summary);
                self.error = None;
                self.fetch_active_tab(false)
            }
            Err(err) => {
                self.loading = false;
                self.error = Some(err);
                Task::none()
            }
        }
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
            Output = Result<Vec<cosmic_containers::ContainerSummary>, String>,
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

fn prune_completed(result: Result<String, String>) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::PruneCompleted(result))
}

fn containers_loaded(
    result: Result<Vec<cosmic_containers::ContainerSummary>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::ContainersLoaded {
        result,
        silent: false,
    })
}

fn images_loaded(
    result: Result<Vec<cosmic_containers::ImageSummary>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::ImagesLoaded {
        result,
        silent: false,
    })
}

fn volumes_loaded(
    result: Result<Vec<cosmic_containers::VolumeSummary>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::VolumesLoaded {
        result,
        silent: false,
    })
}

fn ports_loaded(
    result: Result<Vec<cosmic_containers::PortEntry>, String>,
) -> cosmic::Action<Message> {
    cosmic::Action::App(Message::PortsLoaded {
        result,
        silent: false,
    })
}
