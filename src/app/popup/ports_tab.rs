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
    let ports = state.dev_ports();
    if ports.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No dev servers listening");
    }

    let total = total_pages(ports.len(), LIST_PAGE_SIZE);
    let page = clamp_page(state.tab_pages.ports, total);
    let range = page_range(page, LIST_PAGE_SIZE, ports.len());

    let mut list = widget::column![].spacing(SPACING_NONE);
    for entry in &ports[range] {
        list = list.push(port_row(entry, &state.pending_kill));
    }
    paginated_list(list.into(), page, total)
}

fn port_row<'a>(entry: &'a PortEntry, pending_kill: &Option<PendingKill>) -> Element<'a, Message> {
    if matches!(pending_kill, Some(pending) if Some(pending.pid) == entry.pid) {
        return kill_confirm_row(format!("Kill {}?", entry.kill_label()));
    }

    list_row(
        item_labels_detail(
            entry.title(),
            entry.owner_line(),
            Some(entry.address_line()),
        ),
        port_actions(entry),
    )
}

fn port_actions<'a>(entry: &PortEntry) -> Element<'a, Message> {
    let mut actions = widget::row![].spacing(SPACING_XXS);
    if let (Some(pid), Some(target)) = (entry.pid, entry.signal_target()) {
        actions = actions.push(action_svg_button(
            action_icons::STOP,
            Message::RequestKill(PendingKill {
                pid,
                target,
                label: entry.kill_label(),
            }),
            false,
        ));
    }
    actions.into()
}
