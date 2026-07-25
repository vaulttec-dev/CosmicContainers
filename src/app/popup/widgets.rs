use cosmic::Apply;
use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::theme;
use cosmic::widget;
use cosmic_containers::PopupTab;

use super::Message;
use super::layout::{
    FILL_PORTION_LABELS, FILL_PORTION_PROMPT, FONT_PRIMARY, FONT_SECONDARY, FONT_STATUS, ICON_SIZE,
    PAD_CONFIRM_BUTTON, PAD_ICON_BUTTON, PAD_LIST_ROW, PAD_SECTION_DIVIDER, PAD_STATUS, PAD_TAB,
    POPUP_HEIGHT, POPUP_WIDTH, SPACING_MD, SPACING_NONE, SPACING_SM, SPACING_XXS,
};

pub(super) fn popup_shell<'a>(
    content: impl Into<Element<'a, Message>>,
    core: &cosmic::app::Core,
) -> Element<'a, cosmic::Action<Message>> {
    Element::from(
        core.applet
            .popup_container(content)
            .min_width(POPUP_WIDTH)
            .max_width(POPUP_WIDTH)
            .min_height(POPUP_HEIGHT)
            .max_height(POPUP_HEIGHT)
            .auto_width(false)
            .auto_height(false),
    )
    .map(cosmic::Action::App)
}

pub(super) fn status_line<'a>(text: &'a str) -> Element<'a, Message> {
    widget::container(widget::text(text).size(FONT_STATUS))
        .width(Length::Fill)
        .padding(PAD_STATUS)
        .into()
}

pub(super) fn tab_button<'a>(
    label: &'static str,
    tab: PopupTab,
    active: PopupTab,
) -> Element<'a, Message> {
    let class = if tab == active {
        theme::Button::Suggested
    } else {
        theme::Button::Text
    };
    widget::button::text(label)
        .padding(PAD_TAB)
        .class(class)
        .on_press(Message::TabSelected(tab))
        .into()
}

pub(super) fn action_svg_button<'a>(
    svg: &'static [u8],
    message: Message,
    selected: bool,
) -> Element<'a, Message> {
    widget::button::icon(widget::icon::from_svg_bytes(svg).symbolic(true))
        .padding(PAD_ICON_BUTTON)
        .selected(selected)
        .on_press(message)
        .into()
}

pub(super) fn action_icon_button<'a>(
    icon_name: &'static str,
    message: Message,
) -> Element<'a, Message> {
    widget::icon::from_name(icon_name)
        .size(ICON_SIZE)
        .symbolic(true)
        .apply(widget::button::icon)
        .padding(PAD_ICON_BUTTON)
        .on_press(message)
        .into()
}

pub(super) fn item_labels<'a>(primary: String, secondary: String) -> Element<'a, Message> {
    widget::column![
        widget::text(primary).size(FONT_PRIMARY),
        widget::text(secondary).size(FONT_SECONDARY),
    ]
    .spacing(SPACING_XXS)
    .into()
}

/// Item labels with an extra metadata line (ports, resource usage).
pub(super) fn item_labels_detail<'a>(
    primary: String,
    secondary: String,
    detail: Option<String>,
) -> Element<'a, Message> {
    let mut column = widget::column![
        widget::text(primary).size(FONT_PRIMARY),
        widget::text(secondary).size(FONT_SECONDARY),
    ]
    .spacing(SPACING_XXS);
    if let Some(detail) = detail {
        column = column.push(widget::text(detail).size(FONT_SECONDARY));
    }
    column.into()
}

pub(super) fn list_row<'a>(
    labels: Element<'a, Message>,
    actions: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    widget::container(
        widget::row![
            widget::container(labels).width(Length::FillPortion(FILL_PORTION_LABELS)),
            actions.into(),
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_MD),
    )
    .padding(PAD_LIST_ROW)
    .width(Length::Fill)
    .into()
}

pub(super) fn empty_state<'a>(message: &'static str) -> Element<'a, Message> {
    widget::container(widget::text(message).size(FONT_PRIMARY))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub(super) fn scrollable_list<'a>(list: Element<'a, Message>) -> Element<'a, Message> {
    widget::scrollable(list)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

pub(super) fn paginated_list<'a>(
    list: Element<'a, Message>,
    current_page: usize,
    total_pages: usize,
) -> Element<'a, Message> {
    widget::column![
        scrollable_list(list),
        pagination_bar(current_page, total_pages),
    ]
    .spacing(SPACING_NONE)
    .height(Length::Fill)
    .into()
}

pub(super) fn pagination_bar<'a>(current_page: usize, total_pages: usize) -> Element<'a, Message> {
    let page_label = format!("Page {} / {total_pages}", current_page + 1);
    let at_first = current_page == 0;
    let at_last = current_page + 1 >= total_pages;

    let prev: Element<'a, Message> = if at_first {
        widget::button::text("‹").padding(PAD_TAB).into()
    } else {
        widget::button::text("‹")
            .padding(PAD_TAB)
            .class(theme::Button::Text)
            .on_press(Message::TabPagePrev)
            .into()
    };

    let next: Element<'a, Message> = if at_last {
        widget::button::text("›").padding(PAD_TAB).into()
    } else {
        widget::button::text("›")
            .padding(PAD_TAB)
            .class(theme::Button::Text)
            .on_press(Message::TabPageNext)
            .into()
    };

    widget::container(
        widget::row![
            prev,
            widget::text(page_label).size(FONT_STATUS),
            widget::text("").width(Length::Fill),
            next,
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SM),
    )
    .width(Length::Fill)
    .padding(PAD_STATUS)
    .into()
}

pub(super) fn remove_confirm_row<'a>(prompt: String) -> Element<'a, Message> {
    widget::container(
        widget::row![
            widget::text(prompt)
                .width(Length::FillPortion(FILL_PORTION_PROMPT))
                .size(FONT_PRIMARY),
            widget::button::destructive("Yes")
                .on_press(Message::ConfirmRemove)
                .padding(PAD_CONFIRM_BUTTON),
            widget::button::text("No")
                .on_press(Message::CancelRemove)
                .padding(PAD_CONFIRM_BUTTON),
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SM),
    )
    .padding(PAD_LIST_ROW)
    .into()
}

pub(super) fn prune_confirm_row<'a>(prompt: &'static str) -> Element<'a, Message> {
    widget::container(
        widget::row![
            widget::text(prompt)
                .width(Length::FillPortion(FILL_PORTION_PROMPT))
                .size(FONT_PRIMARY),
            widget::button::destructive("Clean")
                .on_press(Message::ConfirmPrune)
                .padding(PAD_CONFIRM_BUTTON),
            widget::button::text("Cancel")
                .on_press(Message::CancelPrune)
                .padding(PAD_CONFIRM_BUTTON),
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SM),
    )
    .padding(PAD_LIST_ROW)
    .into()
}

pub(super) fn section_divider<'a>() -> Element<'a, Message> {
    widget::container(widget::divider::horizontal::default())
        .padding(PAD_SECTION_DIVIDER)
        .width(Length::Fill)
        .into()
}
