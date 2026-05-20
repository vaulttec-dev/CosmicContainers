use cosmic::app::{Core, Task};
use cosmic::iced::theme::Style;
use cosmic::prelude::*;
use cosmic::widget;
use linux_pop_extension::PANEL_EMOJI;

/// COSMIC panel applet state.
#[derive(Default)]
pub struct App {
    core: Core,
}

/// Messages from widgets (none on the starter applet).
#[derive(Debug, Clone)]
pub enum Message {}

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
        (Self { core }, Task::none())
    }

    fn view(&self) -> Element<'_, Self::Message> {
        widget::text(PANEL_EMOJI).into()
    }

    fn update(&mut self, _message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn style(&self) -> Option<Style> {
        Some(cosmic::applet::style())
    }
}
