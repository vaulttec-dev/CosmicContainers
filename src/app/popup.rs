use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::prelude::*;
use cosmic::widget;
use linux_pop_extension::action_icons;
use linux_pop_extension::icons;
use linux_pop_extension::{
    ContainerGroup, ContainerSummary, ImageSummary, PopupTab, VolumeSummary,
};

use super::{App, Message, PendingRemove};

pub const POPUP_WIDTH: f32 = 380.0;
pub const POPUP_HEIGHT: f32 = 460.0;

const ACTION_ICON_SIZE: u16 = 16;

pub(super) fn popup_view(state: &App) -> Element<'_, cosmic::Action<Message>> {
    let mut content = widget::column![].spacing(6).padding(12);

    let mut toolbar = widget::row![].spacing(2);
    if state.active_tab == PopupTab::Containers {
        toolbar = toolbar.push(action_svg_button(
            action_icons::HIDE,
            Message::ToggleRunningOnly,
        ));
    }
    toolbar = toolbar.push(action_icon_button(icons::REFRESH, Message::Refresh));

    content = content.push(
        widget::row![
            tab_button("Containers", PopupTab::Containers),
            tab_button("Images", PopupTab::Images),
            tab_button("Volumes", PopupTab::Volumes),
            widget::text("").width(Length::Fill),
            toolbar,
        ]
        .align_y(Alignment::Center)
        .spacing(4),
    );

    if state.loading {
        content = content.push(widget::text("Loading..."));
    }

    if let Some(err) = &state.error {
        content = content.push(widget::text(err).size(12));
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

fn action_svg_button<'a>(svg: &'static [u8], message: Message) -> Element<'a, Message> {
    widget::button::icon(widget::icon::from_svg_bytes(svg).symbolic(true))
        .padding(4)
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

fn tab_button<'a>(label: &'static str, tab: PopupTab) -> Element<'a, Message> {
    widget::button::text(label)
        .padding([4, 8])
        .on_press(Message::TabSelected(tab))
        .into()
}

fn remove_confirm_row<'a>(prompt: String) -> Element<'a, Message> {
    widget::row![
        widget::text(prompt).width(Length::FillPortion(2)),
        widget::button::text("Yes")
            .on_press(Message::ConfirmRemove)
            .padding(4),
        widget::button::text("No")
            .on_press(Message::CancelRemove)
            .padding(4),
    ]
    .align_y(Alignment::Center)
    .spacing(6)
    .into()
}

fn container_row<'a>(
    container: &'a ContainerSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let name = container.name.clone();
    let line = format!("{} — {}", container.display_label(), container.state);

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
        ));
    }
    if container.is_running() {
        actions = actions.push(action_svg_button(
            action_icons::STOP,
            Message::Stop(name.clone()),
        ));
    }
    actions = actions.push(action_svg_button(
        action_icons::RELOAD,
        Message::Restart(name.clone()),
    ));
    if !container.is_running() {
        actions = actions.push(action_svg_button(
            action_icons::DELETE,
            Message::RequestRemove(PendingRemove::Container(name)),
        ));
    }

    widget::row![widget::text(line).width(Length::FillPortion(3)), actions]
        .align_y(Alignment::Center)
        .spacing(8)
        .into()
}

fn group_section<'a>(state: &'a App, group: &'a ContainerGroup) -> Element<'a, Message> {
    let running = group.running_count();
    let total = group.containers.len();
    let header = format!(
        "{}  ({running}/{total} running)",
        group.project,
        running = running,
        total = total
    );

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
            ));
        }
        if group.has_running() {
            group_actions = group_actions.push(action_svg_button(
                action_icons::STOP,
                Message::StopGroup(project.clone()),
            ));
            group_actions = group_actions.push(action_svg_button(
                action_icons::RELOAD,
                Message::RestartGroup(project),
            ));
        }
    }

    let header_row = widget::row![
        action_icon_button(
            collapse_icon,
            Message::ToggleGroupCollapse(group.project.clone())
        ),
        widget::text(header).width(Length::FillPortion(2)),
        group_actions,
    ]
    .align_y(Alignment::Center)
    .spacing(6);

    let mut section = widget::column![header_row].spacing(4);
    if !collapsed {
        for container in &group.containers {
            section = section.push(container_row(container, &state.pending_remove));
        }
    }
    section.into()
}

fn containers_tab<'a>(state: &'a App) -> Element<'a, Message> {
    let mut content = widget::column![].spacing(6);

    if state.groups.is_empty() && !state.loading && state.error.is_none() {
        content = content.push(widget::text("No containers."));
        return content.into();
    }

    let mut list = widget::column![].spacing(10);
    for (index, group) in state.groups.iter().enumerate() {
        if index > 0 {
            list = list.push(widget::divider::horizontal::default());
        }
        list = list.push(group_section(state, group));
    }
    content = content.push(
        widget::scrollable(list)
            .height(Length::Fill)
            .width(Length::Fill),
    );
    content.into()
}

fn image_row<'a>(
    image: &'a ImageSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let id = image.id.clone();
    let line = format!("{} — {} ({})", image.reference, image.size, image.id);

    if matches!(pending_remove, Some(PendingRemove::Image(i)) if i == &id) {
        return remove_confirm_row(format!("Remove image {id}?"));
    }

    let actions = widget::row![action_svg_button(
        action_icons::DELETE,
        Message::RequestRemove(PendingRemove::Image(id)),
    )]
    .spacing(2);

    widget::row![
        widget::text(line).width(Length::FillPortion(3)).size(12),
        actions
    ]
    .align_y(Alignment::Center)
    .spacing(8)
    .into()
}

fn volume_row<'a>(
    volume: &'a VolumeSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let name = volume.name.clone();
    let line = format!("{} — {}", volume.name, volume.driver);

    if matches!(pending_remove, Some(PendingRemove::Volume(n)) if n == &name) {
        return remove_confirm_row(format!("Remove volume {name}?"));
    }

    let actions = widget::row![action_svg_button(
        action_icons::DELETE,
        Message::RequestRemove(PendingRemove::Volume(name)),
    )]
    .spacing(2);

    widget::row![
        widget::text(line).width(Length::FillPortion(3)).size(12),
        actions
    ]
    .align_y(Alignment::Center)
    .spacing(8)
    .into()
}

fn images_tab<'a>(state: &'a App) -> Element<'a, Message> {
    let mut content = widget::column![].spacing(4);
    if state.image_list.is_empty() && !state.loading && state.error.is_none() {
        content = content.push(widget::text("No images."));
        return content.into();
    }
    for image in &state.image_list {
        content = content.push(image_row(image, &state.pending_remove));
    }
    widget::scrollable(content)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn volumes_tab<'a>(state: &'a App) -> Element<'a, Message> {
    let mut content = widget::column![].spacing(4);
    if state.volume_list.is_empty() && !state.loading && state.error.is_none() {
        content = content.push(widget::text("No volumes."));
        return content.into();
    }
    for volume in &state.volume_list {
        content = content.push(volume_row(volume, &state.pending_remove));
    }
    widget::scrollable(content)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}
