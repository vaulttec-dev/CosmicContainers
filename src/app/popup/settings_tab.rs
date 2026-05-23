use cosmic::Element;
use cosmic::iced::alignment::Alignment;
use cosmic::widget;
use linux_pop_extension::{
    DEFAULT_PANEL_COUNT_FONT_SIZE, DOCKER_ICON_SVG, MAX_PANEL_COUNT_FONT_SIZE,
    MIN_PANEL_COUNT_FONT_SIZE,
};

use super::layout::{FONT_PRIMARY, PAD_TAB, SPACING_MD, SPACING_SM};
use super::widgets::item_labels;
use super::{App, Message};

pub(super) fn settings_tab<'a>(state: &'a App) -> Element<'a, Message> {
    let size = state.settings.panel_count_font_size;
    let preview = widget::row![
        widget::icon(widget::icon::from_svg_bytes(DOCKER_ICON_SVG)).size(16),
        widget::text(state.panel_label()).size(size),
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    widget::column![
        item_labels(
            "Running count font size".to_string(),
            format!("Panel badge ({MIN_PANEL_COUNT_FONT_SIZE}–{MAX_PANEL_COUNT_FONT_SIZE})"),
        ),
        widget::spin_button(
            format!("{size} pt"),
            "Running count font size",
            size as i32,
            1,
            i32::from(MIN_PANEL_COUNT_FONT_SIZE),
            i32::from(MAX_PANEL_COUNT_FONT_SIZE),
            |value| Message::PanelCountFontSizeChanged(value as u16),
        ),
        widget::text("Preview").size(FONT_PRIMARY),
        preview,
        widget::button::text("Reset to default")
            .on_press(Message::PanelCountFontSizeChanged(
                DEFAULT_PANEL_COUNT_FONT_SIZE,
            ))
            .padding(PAD_TAB),
    ]
    .spacing(SPACING_MD)
    .into()
}
