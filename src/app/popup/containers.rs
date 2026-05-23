use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::widget;
use linux_pop_extension::action_icons;
use linux_pop_extension::icons;
use linux_pop_extension::{ContainerGroup, ContainerSummary};

use super::layout::{FILL_PORTION_PROMPT, PAD_GROUP_HEADER, SPACING_NONE, SPACING_SM, SPACING_XXS};
use super::widgets::{
    action_icon_button, action_svg_button, empty_state, item_labels, list_row, remove_confirm_row,
    scrollable_list, section_divider,
};
use super::{App, Message};
use crate::app::PendingRemove;

pub(super) fn containers_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.groups.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("Немає контейнерів");
    }

    let mut list = widget::column![].spacing(SPACING_NONE);
    for (index, group) in state.groups.iter().enumerate() {
        if index > 0 {
            list = list.push(section_divider());
        }
        list = list.push(group_section(state, group));
    }
    scrollable_list(list.into())
}

fn container_state_line(container: &ContainerSummary) -> String {
    if container.is_running() {
        format!("● {}", container.state)
    } else {
        container.state.clone()
    }
}

fn container_actions(name: String, running: bool) -> Element<'static, Message> {
    let mut actions = widget::row![].spacing(SPACING_XXS);
    if !running {
        actions = actions.push(action_svg_button(
            action_icons::PLAY,
            Message::Start(name.clone()),
            false,
        ));
    } else {
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
    if !running {
        actions = actions.push(action_svg_button(
            action_icons::DELETE,
            Message::RequestRemove(PendingRemove::Container(name)),
            false,
        ));
    }
    actions.into()
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
        return remove_confirm_row(format!("Видалити {name}?"));
    }

    list_row(
        item_labels(container.display_label(), container_state_line(container)),
        container_actions(name, container.is_running()),
    )
}

fn group_toolbar<'a>(group: &'a ContainerGroup, collapsed: bool) -> Element<'a, Message> {
    let project = group.project.clone();
    let mut actions = widget::row![].spacing(SPACING_XXS);
    if !collapsed {
        if group.has_stopped() {
            actions = actions.push(action_svg_button(
                action_icons::PLAY,
                Message::StartGroup(project.clone()),
                false,
            ));
        }
        if group.has_running() {
            actions = actions.push(action_svg_button(
                action_icons::STOP,
                Message::StopGroup(project.clone()),
                false,
            ));
            actions = actions.push(action_svg_button(
                action_icons::RELOAD,
                Message::RestartGroup(project),
                false,
            ));
        }
    }
    actions.into()
}

fn group_header<'a>(
    group: &'a ContainerGroup,
    collapsed: bool,
    toolbar: Element<'a, Message>,
) -> Element<'a, Message> {
    let running = group.running_count();
    let total = group.containers.len();
    let collapse_icon = if collapsed {
        icons::EXPAND
    } else {
        icons::COLLAPSE
    };

    widget::container(
        widget::row![
            action_icon_button(
                collapse_icon,
                Message::ToggleGroupCollapse(group.project.clone()),
            ),
            widget::container(item_labels(
                group.project.clone(),
                format!("{running}/{total} запущено"),
            ))
            .width(Length::FillPortion(FILL_PORTION_PROMPT)),
            toolbar,
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SM),
    )
    .padding(PAD_GROUP_HEADER)
    .width(Length::Fill)
    .into()
}

fn group_section<'a>(state: &'a App, group: &'a ContainerGroup) -> Element<'a, Message> {
    let collapsed = state.collapsed_groups.contains(&group.project);
    let header = group_header(group, collapsed, group_toolbar(group, collapsed));

    let mut section = widget::column![header].spacing(SPACING_NONE);
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
