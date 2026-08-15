use cosmic::Element;
use cosmic::widget;
use cosmic_containers::PortEntry;
use cosmic_containers::action_icons;
use cosmic_containers::{LIST_PAGE_SIZE, clamp_page, page_range, total_pages};

use super::layout::SPACING_NONE;
use super::layout::SPACING_XXS;
use super::widgets::{
    action_svg_button, empty_state, item_labels_detail, kill_confirm_row, list_row, paginated_list,
};
use super::{App, Message};
use crate::app::PendingKill;

pub(super) fn ports_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.port_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No listening ports");
    }

    let total = total_pages(state.port_list.len(), LIST_PAGE_SIZE);
    let page = clamp_page(state.tab_pages.ports, total);
    let range = page_range(page, LIST_PAGE_SIZE, state.port_list.len());

    let mut list = widget::column![].spacing(SPACING_NONE);
    for entry in &state.port_list[range] {
        list = list.push(port_row(
            entry,
            state.container_for_port(entry.port),
            &state.pending_kill,
        ));
    }
    paginated_list(list.into(), page, total)
}

fn port_row<'a>(
    entry: &'a PortEntry,
    container: Option<&'a str>,
    pending_kill: &Option<PendingKill>,
) -> Element<'a, Message> {
    if matches!(pending_kill, Some(pending) if Some(pending.pid) == entry.pid) {
        return kill_confirm_row(format!("Kill {}?", entry.kill_label()));
    }

    list_row(
        item_labels_detail(
            entry.title(),
            entry.owner_line(container),
            Some(entry.address_line()),
        ),
        port_actions(entry, container),
    )
}

/// A published port is freed by stopping its container; anything else by signalling the PID.
fn port_actions<'a>(entry: &PortEntry, container: Option<&str>) -> Element<'a, Message> {
    let mut actions = widget::row![].spacing(SPACING_XXS);
    if let Some(container) = container {
        actions = actions.push(action_svg_button(
            action_icons::STOP,
            Message::StopContainerPort(container.to_string()),
            false,
        ));
        return actions.into();
    }
    if let Some(pid) = entry.pid {
        actions = actions.push(action_svg_button(
            action_icons::STOP,
            Message::RequestKill(PendingKill {
                pid,
                label: entry.kill_label(),
            }),
            false,
        ));
    }
    actions.into()
}
