use cosmic::app::{Core, Task};
use cosmic::iced::theme::Style;
use cosmic::iced::window;
use cosmic::iced::window::Id;
use cosmic::prelude::*;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use linux_pop_extension::{PANEL_EMOJI, POPUP_GREETING};

/// COSMIC panel applet state.
#[derive(Default)]
pub struct App {
    core: Core,
    popup: Option<Id>,
}

/// Messages from the panel button and popup lifecycle.
#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    TogglePopup,
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
        (Self { core, popup: None }, Task::none())
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
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(app_popup::<App>(
                        |state: &mut App| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            )
                        },
                        Some(Box::new(popup_view)),
                    )),
                ));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        widget::button::text(PANEL_EMOJI)
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
    let content = widget::column![
        widget::text(PANEL_EMOJI).size(28),
        widget::text(POPUP_GREETING),
    ]
    .spacing(8)
    .padding(12);

    Element::from(state.core.applet.popup_container(content)).map(cosmic::Action::App)
}
