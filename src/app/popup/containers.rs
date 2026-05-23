use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced::alignment::Alignment;
use cosmic::widget;
use linux_pop_extension::action_icons;
use linux_pop_extension::icons;
use linux_pop_extension::{
    ContainerGroup, ContainerSummary, LIST_PAGE_SIZE, clamp_page, page_range, total_pages,
};

use super::layout::{FILL_PORTION_PROMPT, PAD_GROUP_HEADER, SPACING_NONE, SPACING_SM, SPACING_XXS};
use super::widgets::{
    action_icon_button, action_svg_button, empty_state, item_labels, list_row, paginated_list,
    remove_confirm_row, section_divider,
};
use super::{App, Message};
use crate::app::PendingRemove;

enum ContainerListRow<'a> {
    GroupHeader {
        group: &'a ContainerGroup,
        collapsed: bool,
    },
    Container {
        container: &'a ContainerSummary,
    },
}

pub(super) fn containers_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.groups.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No containers");
    }

    let rows = build_container_rows(state);
    let total = total_pages(rows.len(), LIST_PAGE_SIZE);
    let page = clamp_page(state.tab_pages.containers, total);
    let range = page_range(page, LIST_PAGE_SIZE, rows.len());

    let mut list = widget::column![].spacing(SPACING_NONE);
    for (index, row) in rows[range].iter().enumerate() {
        if index > 0 && matches!(row, ContainerListRow::GroupHeader { .. }) {
            list = list.push(section_divider());
        }
        match row {
            ContainerListRow::GroupHeader { group, collapsed } => {
                list = list.push(group_header_row(
                    group,
                    *collapsed,
                    group_toolbar(group, *collapsed),
                ));
            }
            ContainerListRow::Container { container } => {
                list = list.push(widget::container(container_row(
                    container,
                    &state.pending_remove,
                )));
            }
        }
    }

    paginated_list(list.into(), page, total)
}

fn build_container_rows<'a>(state: &'a App) -> Vec<ContainerListRow<'a>> {
    let mut rows = Vec::new();
    for group in &state.groups {
        let collapsed = state.collapsed_groups.contains(&group.project);
        rows.push(ContainerListRow::GroupHeader { group, collapsed });
        if !collapsed {
            for container in &group.containers {
                rows.push(ContainerListRow::Container { container });
            }
        }
    }
    rows
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
        return remove_confirm_row(format!("Remove {name}?"));
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

fn group_header_row<'a>(
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
                format!("{running}/{total} running"),
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
