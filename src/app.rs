use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::iced::theme::Style;
use cosmic::iced::window;
use cosmic::iced::window::Id;
use cosmic::prelude::*;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use linux_pop_extension::services::docker;
use linux_pop_extension::{ContainerSummary, running_count};

/// COSMIC panel applet state.
#[derive(Default)]
pub struct App {
    core: Core,
    popup: Option<Id>,
    containers: Vec<ContainerSummary>,
    loading: bool,
    error: Option<String>,
}

/// Messages from the panel button, popup, and Docker CLI results.
#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    TogglePopup,
    Refresh,
    ContainersLoaded(Result<Vec<ContainerSummary>, String>),
    Start(String),
    Stop(String),
}

impl App {
    fn panel_label(&self) -> String {
        if self.error.is_some() && self.containers.is_empty() {
            return "!".to_string();
        }
        format!("{}", running_count(&self.containers))
    }

    fn fetch_containers_task() -> Task<Message> {
        Task::perform(docker::list_containers(), |result| {
            cosmic::Action::App(Message::ContainersLoaded(result))
        })
    }

    fn apply_loaded(&mut self, result: Result<Vec<ContainerSummary>, String>) {
        self.loading = false;
        match result {
            Ok(containers) => {
                self.containers = containers;
                self.error = None;
            }
            Err(err) => {
                self.error = Some(err);
            }
        }
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
                ..Default::default()
            },
            Self::fetch_containers_task(),
        )
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::TogglePopup => {
                if let Some(id) = self.popup.take() {
                    return cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }
                self.loading = true;
                let fetch = Self::fetch_containers_task();
                let open = cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(app_popup::<App>(
                        |state: &mut App| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                Some((360, 420)),
                                None,
                                None,
                            )
                        },
                        Some(Box::new(popup_view)),
                    )),
                ));
                return Task::batch([open, fetch]);
            }
            Message::Refresh => {
                self.loading = true;
                return Self::fetch_containers_task();
            }
            Message::ContainersLoaded(result) => {
                self.apply_loaded(result);
            }
            Message::Start(name) => {
                self.loading = true;
                return Task::perform(docker::start_then_list(name), |result| {
                    cosmic::Action::App(Message::ContainersLoaded(result))
                });
            }
            Message::Stop(name) => {
                self.loading = true;
                return Task::perform(docker::stop_then_list(name), |result| {
                    cosmic::Action::App(Message::ContainersLoaded(result))
                });
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        widget::button::text(self.panel_label())
            .padding(4)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        widget::text("").into()
    }

    fn style(&self) -> Option<Style> {
        Some(cosmic::applet::style())
    }
}

fn popup_view(state: &App) -> Element<'_, cosmic::Action<Message>> {
    let mut content = widget::column![].spacing(6).padding(12);

    content = content.push(
        widget::row![
            widget::text("Containers").size(18),
            widget::button::text("Refresh").on_press(Message::Refresh),
        ]
        .align_y(Alignment::Center)
        .spacing(12),
    );

    if state.loading {
        content = content.push(widget::text("Loading..."));
    }

    if let Some(err) = &state.error {
        content = content.push(widget::text(err).size(12));
    }

    if state.containers.is_empty() && !state.loading && state.error.is_none() {
        content = content.push(widget::text("No containers."));
    }

    for container in &state.containers {
        let name = container.name.clone();
        let line = format!("{} — {}", container.name, container.state);
        let row = if container.is_running() {
            widget::row![
                widget::text(line).width(Length::FillPortion(3)),
                widget::button::text("Stop").on_press(Message::Stop(name)),
            ]
        } else {
            widget::row![
                widget::text(line).width(Length::FillPortion(3)),
                widget::button::text("Play").on_press(Message::Start(name)),
            ]
        };
        content = content.push(row.align_y(Alignment::Center).spacing(8));
    }

    Element::from(
        state
            .core
            .applet
            .popup_container(content)
            .max_width(360.0)
            .max_height(420.0),
    )
    .map(cosmic::Action::App)
}
