use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::theme;
use cosmic::widget;

use super::{App, Message};

/// Gap between the Docker mark and the running count.
const PANEL_SPACING: u16 = 6;

/// Rendered text is roughly this much taller than its font size.
const LINE_HEIGHT_RATIO: f32 = 1.3;

pub(super) fn panel_view(app: &App) -> Element<'_, Message> {
    let applet = &app.core.applet;
    let (icon_size, suggested_size) = applet.suggested_size(false);
    let (padding_major, padding_minor) = applet.suggested_padding(true);
    let horizontal = applet.is_horizontal();
    let (horizontal_padding, vertical_padding) = if horizontal {
        (padding_major, padding_minor)
    } else {
        (padding_minor, padding_major)
    };
    let button_width = f32::from(icon_size) + f32::from(horizontal_padding) * 2.0;
    let button_height = f32::from(suggested_size) + f32::from(vertical_padding) * 2.0;

    // The panel fixes the applet's minor axis, so cap the font to what fits there.
    let minor_axis = if horizontal {
        button_height
    } else {
        button_width
    };
    let font_size =
        f32::from(app.settings.panel_count_font_size).min(minor_axis / LINE_HEIGHT_RATIO);

    let docker_icon = widget::icon(widget::icon::from_svg_bytes(
        cosmic_containers::APP_ICON_SVG,
    ))
    .size(icon_size);
    let label = widget::text(app.panel_label()).size(font_size);
    let panel_content: Element<'_, Message> = if horizontal {
        widget::row![docker_icon, label]
            .spacing(PANEL_SPACING)
            .align_y(Alignment::Center)
            .into()
    } else {
        widget::column![docker_icon, label]
            .spacing(PANEL_SPACING)
            .align_x(Alignment::Center)
            .into()
    };

    // Mirrors `applet::text_button`: only the panel's minor axis is fixed, so the count
    // is never clipped. (`applet::button_from_element` fixes both axes to a single icon.)
    let (content, padding) = if horizontal {
        (
            widget::layer_container(panel_content).center_y(Length::Fixed(button_height)),
            [0, horizontal_padding],
        )
    } else {
        (
            widget::layer_container(panel_content).center_x(Length::Fixed(button_width)),
            [vertical_padding, 0],
        )
    };
    let button = widget::button::custom(content)
        .padding(padding)
        .class(theme::Button::AppletIcon)
        .on_press(Message::TogglePopup);

    // The applet surface itself is fixed-size unless wrapped: without this the count is
    // laid out beyond the window bounds and disappears entirely.
    applet.autosize_window(button).into()
}

pub(super) fn empty_window() -> Element<'static, Message> {
    widget::text("").into()
}
