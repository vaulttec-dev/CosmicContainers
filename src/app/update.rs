use cosmic::app::Task;
use cosmic::surface::action::{app_popup, destroy_popup};
use linux_pop_extension::PopupTab;
use linux_pop_extension::services::docker;

use super::popup::popup_view;
use super::{App, Message, POPUP_HEIGHT, POPUP_WIDTH};

impl App {
    pub(super) fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => self.on_popup_closed(id),
            Message::TogglePopup => self.toggle_popup(),
            Message::Poll => self.poll_lists(),
            Message::Refresh => self.refresh_active_tab(),
            Message::TabSelected(tab) => self.select_tab(tab),
            Message::ToggleRunningOnly => self.toggle_running_filter(),
            Message::ToggleGroupCollapse(project) => self.toggle_group_collapse(project),
            Message::RequestRemove(name) => self.request_remove(name),
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
        if self.popup.is_some() && self.active_tab != PopupTab::Containers {
            tasks.push(Self::fetch_tab_task(self.active_tab, silent));
        }
        Task::batch(tasks)
    }

    fn refresh_active_tab(&mut self) -> Task<Message> {
        self.loading = true;
        self.fetch_active_tab(false)
    }

    fn select_tab(&mut self, tab: PopupTab) -> Task<Message> {
        if self.active_tab == tab {
            return Task::none();
        }
        self.active_tab = tab;
        self.loading = true;
        self.error = None;
        self.fetch_active_tab(false)
    }

    fn toggle_running_filter(&mut self) -> Task<Message> {
        self.running_only = !self.running_only;
        self.rebuild_groups();
        Task::none()
    }

    fn toggle_group_collapse(&mut self, project: String) -> Task<Message> {
        if self.collapsed_groups.contains(&project) {
            self.collapsed_groups.remove(&project);
        } else {
            self.collapsed_groups.insert(project);
        }
        Task::none()
    }

    fn request_remove(&mut self, name: String) -> Task<Message> {
        self.pending_remove = Some(name);
        Task::none()
    }

    fn cancel_remove(&mut self) -> Task<Message> {
        self.pending_remove = None;
        Task::none()
    }

    fn confirm_remove(&mut self) -> Task<Message> {
        let Some(name) = self.pending_remove.take() else {
            return Task::none();
        };
        self.loading = true;
        Task::perform(docker::remove_then_list(name), |result| {
            cosmic::Action::App(Message::ContainersLoaded {
                result,
                silent: false,
            })
        })
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
