use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget;
use linux_pop_extension::action_icons;
use linux_pop_extension::icons;
use linux_pop_extension::{
    ContainerGroup, ContainerSummary, ImageSummary, PopupTab, VolumeSummary,
};

use super::{App, Message, PendingRemove};

pub const POPUP_WIDTH: f32 = 400.0;
pub const POPUP_HEIGHT: f32 = 480.0;

const ACTION_ICON_SIZE: u16 = 16;

pub(super) fn popup_view(state: &App) -> Element<'_, cosmic::Action<Message>> {
    let mut content = widget::column![].spacing(0).padding([10, 12, 12, 12]);

    content = content.push(header_bar(state));
    content = content.push(
        widget::container(widget::divider::horizontal::default())
            .padding([6, 0])
            .width(Length::Fill),
    );

    if state.loading {
        content = content.push(status_line("Loading…"));
    }

    if let Some(err) = &state.error {
        content = content.push(status_line(err.as_str()));
    }

    let body = match state.active_tab {
        PopupTab::Containers => containers_tab(state),
        PopupTab::Images => images_tab(state),
        PopupTab::Volumes => volumes_tab(state),
    };
    content = content.push(body);

    Element::from(
        state
            .core
            .applet
            .popup_container(content)
            .max_width(POPUP_WIDTH)
            .max_height(POPUP_HEIGHT),
    )
    .map(cosmic::Action::App)
}

fn header_bar<'a>(state: &'a App) -> Element<'a, Message> {
    let mut toolbar = widget::row![].spacing(4).align_y(Alignment::Center);
    if state.active_tab == PopupTab::Containers {
        toolbar = toolbar.push(action_svg_button(
            action_icons::HIDE,
            Message::ToggleRunningOnly,
            state.running_only,
        ));
    }
    toolbar = toolbar.push(action_svg_button(
        action_icons::RELOAD,
        Message::Refresh,
        false,
    ));

    widget::row![
        tab_button("Containers", PopupTab::Containers, state.active_tab),
        tab_button("Images", PopupTab::Images, state.active_tab),
        tab_button("Volumes", PopupTab::Volumes, state.active_tab),
        widget::text("").width(Length::Fill),
        toolbar,
    ]
    .align_y(Alignment::Center)
    .spacing(6)
    .into()
}

fn status_line<'a>(text: &'a str) -> Element<'a, Message> {
    widget::container(widget::text(text).size(12))
        .width(Length::Fill)
        .padding([0, 0, 4, 0])
        .into()
}

fn tab_button<'a>(label: &'static str, tab: PopupTab, active: PopupTab) -> Element<'a, Message> {
    let class = if tab == active {
        theme::Button::Suggested
    } else {
        theme::Button::Text
    };
    widget::button::text(label)
        .padding([6, 10])
        .class(class)
        .on_press(Message::TabSelected(tab))
        .into()
}

fn action_svg_button<'a>(
    svg: &'static [u8],
    message: Message,
    selected: bool,
) -> Element<'a, Message> {
    widget::button::icon(widget::icon::from_svg_bytes(svg).symbolic(true))
        .padding(4)
        .selected(selected)
        .on_press(message)
        .into()
}

fn action_icon_button<'a>(icon_name: &'static str, message: Message) -> Element<'a, Message> {
    widget::icon::from_name(icon_name)
        .size(ACTION_ICON_SIZE)
        .symbolic(true)
        .apply(widget::button::icon)
        .padding(4)
        .on_press(message)
        .into()
}

fn item_labels<'a>(primary: String, secondary: String) -> Element<'a, Message> {
    widget::column![
        widget::text(primary).size(13),
        widget::text(secondary).size(11),
    ]
    .spacing(2)
    .into()
}

fn list_row<'a>(
    labels: Element<'a, Message>,
    actions: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    widget::container(
        widget::row![
            widget::container(labels).width(Length::FillPortion(3)),
            actions.into(),
        ]
        .align_y(Alignment::Center)
        .spacing(8),
    )
    .padding([6, 4])
    .width(Length::Fill)
    .into()
}

fn empty_state<'a>(message: &'static str) -> Element<'a, Message> {
    widget::container(widget::text(message).size(13))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn scrollable_list<'a>(list: Element<'a, Message>) -> Element<'a, Message> {
    widget::scrollable(list)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn remove_confirm_row<'a>(prompt: String) -> Element<'a, Message> {
    widget::container(
        widget::row![
            widget::text(prompt).width(Length::FillPortion(2)).size(13),
            widget::button::destructive("Yes")
                .on_press(Message::ConfirmRemove)
                .padding([4, 8]),
            widget::button::text("No")
                .on_press(Message::CancelRemove)
                .padding([4, 8]),
        ]
        .align_y(Alignment::Center)
        .spacing(6),
    )
    .padding([6, 4])
    .into()
}

fn container_state_line(container: &ContainerSummary) -> String {
    if container.is_running() {
        format!("● {}", container.state)
    } else {
        container.state.clone()
    }
}

fn container_row<'a>(
    container: &'a ContainerSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let name = container.name.clone();

    if matches!(
        pending_remove,
        Some(PendingRemove::Container(n)) if n == &name
    ) {
        return remove_confirm_row(format!("Remove {name}?"));
    }

    let mut actions = widget::row![].spacing(2);
    if !container.is_running() {
        actions = actions.push(action_svg_button(
            action_icons::PLAY,
            Message::Start(name.clone()),
            false,
        ));
    }
    if container.is_running() {
        actions = actions.push(action_svg_button(
            action_icons::STOP,
            Message::Stop(name.clone()),
            false,
        ));
    }
    actions = actions.push(action_svg_button(
        action_icons::RELOAD,
        Message::Restart(name.clone()),
        false,
    ));
    if !container.is_running() {
        actions = actions.push(action_svg_button(
            action_icons::DELETE,
            Message::RequestRemove(PendingRemove::Container(name)),
            false,
        ));
    }

    list_row(
        item_labels(container.display_label(), container_state_line(container)),
        actions,
    )
}

fn group_section<'a>(state: &'a App, group: &'a ContainerGroup) -> Element<'a, Message> {
    let running = group.running_count();
    let total = group.containers.len();
    let collapsed = state.collapsed_groups.contains(&group.project);
    let collapse_icon = if collapsed {
        icons::EXPAND
    } else {
        icons::COLLAPSE
    };
    let project = group.project.clone();

    let mut group_actions = widget::row![].spacing(2);
    if !collapsed {
        if group.has_stopped() {
            group_actions = group_actions.push(action_svg_button(
                action_icons::PLAY,
                Message::StartGroup(project.clone()),
                false,
            ));
        }
        if group.has_running() {
            group_actions = group_actions.push(action_svg_button(
                action_icons::STOP,
                Message::StopGroup(project.clone()),
                false,
            ));
            group_actions = group_actions.push(action_svg_button(
                action_icons::RELOAD,
                Message::RestartGroup(project),
                false,
            ));
        }
    }

    let header_row = widget::container(
        widget::row![
            action_icon_button(
                collapse_icon,
                Message::ToggleGroupCollapse(group.project.clone())
            ),
            widget::container(item_labels(
                group.project.clone(),
                format!("{running}/{total} running"),
            ))
            .width(Length::FillPortion(2)),
            Element::from(group_actions),
        ]
        .align_y(Alignment::Center)
        .spacing(6),
    )
    .padding([8, 4, 4, 4])
    .width(Length::Fill);

    let mut section = widget::column![header_row].spacing(0);
    if !collapsed {
        for container in &group.containers {
            section = section.push(widget::container(container_row(
                container,
                &state.pending_remove,
            )));
        }
    }
    section.into()
}

fn containers_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.groups.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No containers");
    }

    let mut list = widget::column![].spacing(0);
    for (index, group) in state.groups.iter().enumerate() {
        if index > 0 {
            list = list.push(
                widget::container(widget::divider::horizontal::default())
                    .padding([4, 8])
                    .width(Length::Fill),
            );
        }
        list = list.push(group_section(state, group));
    }
    scrollable_list(list.into())
}

fn image_row<'a>(
    image: &'a ImageSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let id = image.id.clone();

    if matches!(pending_remove, Some(PendingRemove::Image(i)) if i == &id) {
        return remove_confirm_row(format!("Remove image {id}?"));
    }

    let actions = widget::row![action_svg_button(
        action_icons::DELETE,
        Message::RequestRemove(PendingRemove::Image(id)),
        false,
    )]
    .spacing(2);

    list_row(
        item_labels(
            image.reference.clone(),
            format!("{} · {}", image.size, image.id),
        ),
        actions,
    )
}

fn volume_row<'a>(
    volume: &'a VolumeSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let name = volume.name.clone();

    if matches!(pending_remove, Some(PendingRemove::Volume(n)) if n == &name) {
        return remove_confirm_row(format!("Remove volume {name}?"));
    }

    let actions = widget::row![action_svg_button(
        action_icons::DELETE,
        Message::RequestRemove(PendingRemove::Volume(name)),
        false,
    )]
    .spacing(2);

    list_row(
        item_labels(volume.name.clone(), volume.driver.clone()),
        actions,
    )
}

fn images_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.image_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No images");
    }

    let mut list = widget::column![].spacing(0);
    for image in &state.image_list {
        list = list.push(image_row(image, &state.pending_remove));
    }
    scrollable_list(list.into())
}

fn volumes_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.volume_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No volumes");
    }

    let mut list = widget::column![].spacing(0);
    for volume in &state.volume_list {
        list = list.push(volume_row(volume, &state.pending_remove));
    }
    scrollable_list(list.into())
}
