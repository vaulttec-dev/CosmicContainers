use cosmic::Element;
use cosmic::iced::alignment::Alignment;
use cosmic::widget;

use super::{App, Message};

pub(super) fn panel_view(app: &App) -> Element<'_, Message> {
    let (icon_size, _) = app.core.applet.suggested_size(false);
    let docker_icon = widget::icon(widget::icon::from_svg_bytes(
        linux_pop_extension::DOCKER_ICON_SVG,
    ))
    .size(icon_size);
    let panel_content = widget::row![
        docker_icon,
        widget::text(app.panel_label()).size(app.settings.panel_count_font_size),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    app.core
        .applet
        .button_from_element(panel_content, false)
        .on_press(Message::TogglePopup)
        .into()
}

pub(super) fn empty_window() -> Element<'static, Message> {
    widget::text("").into()
}
