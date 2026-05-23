mod containers;
mod images_tab;
mod layout;
mod settings_tab;
mod volumes_tab;
mod widgets;

use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::widget;
use linux_pop_extension::PopupTab;
use linux_pop_extension::action_icons;

use super::App;
use super::Message;
use layout::{PAD_DIVIDER, PAD_POPUP, SPACING_NONE, SPACING_SM, SPACING_XS};
use widgets::{action_svg_button, popup_shell, status_line, tab_button};

pub use layout::{POPUP_HEIGHT, POPUP_WIDTH};

pub(super) fn popup_view(state: &App) -> Element<'_, cosmic::Action<Message>> {
    let mut content = widget::column![]
        .spacing(SPACING_NONE)
        .padding(PAD_POPUP)
        .width(Length::Fixed(layout::POPUP_WIDTH))
        .height(Length::Fixed(layout::POPUP_HEIGHT));

    content = content.push(header_bar(state));
    content = content.push(
        widget::container(widget::divider::horizontal::default())
            .padding(PAD_DIVIDER)
            .width(Length::Fill),
    );

    if state.loading {
        content = content.push(status_line("Завантаження…"));
    }

    if let Some(err) = &state.error {
        content = content.push(status_line(err.as_str()));
    }

    let tab_body = match state.active_tab {
        PopupTab::Containers => containers::containers_tab(state),
        PopupTab::Images => images_tab::images_tab(state),
        PopupTab::Volumes => volumes_tab::volumes_tab(state),
        PopupTab::Settings => settings_tab::settings_tab(state),
    };
    content = content.push(
        widget::container(tab_body)
            .width(Length::Fill)
            .height(Length::Fill),
    );

    popup_shell(content, &state.core)
}

fn header_bar<'a>(state: &'a App) -> Element<'a, Message> {
    let toolbar = widget::row![action_svg_button(
        action_icons::RELOAD,
        Message::Refresh,
        false,
    ),]
    .spacing(SPACING_XS)
    .align_y(Alignment::Center);

    widget::row![
        tab_button("Контейнери", PopupTab::Containers, state.active_tab),
        tab_button("Образи", PopupTab::Images, state.active_tab),
        tab_button("Томи", PopupTab::Volumes, state.active_tab),
        tab_button("Налаштування", PopupTab::Settings, state.active_tab),
        widget::text("").width(Length::Fill),
        toolbar,
    ]
    .align_y(Alignment::Center)
    .spacing(SPACING_SM)
    .into()
}
