use cosmic::Element;
use cosmic::widget;
use cosmic_containers::VolumeSummary;
use cosmic_containers::action_icons;
use cosmic_containers::{LIST_PAGE_SIZE, clamp_page, page_range, total_pages};

use super::layout::SPACING_NONE;
use super::layout::SPACING_XXS;
use super::widgets::{
    action_svg_button, empty_state, item_labels, list_row, paginated_list, remove_confirm_row,
};
use super::{App, Message};
use crate::app::PendingRemove;

pub(super) fn volumes_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.volume_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No volumes");
    }

    let total = total_pages(state.volume_list.len(), LIST_PAGE_SIZE);
    let page = clamp_page(state.tab_pages.volumes, total);
    let range = page_range(page, LIST_PAGE_SIZE, state.volume_list.len());

    let mut list = widget::column![].spacing(SPACING_NONE);
    for volume in &state.volume_list[range] {
        list = list.push(volume_row(volume, &state.pending_remove));
    }
    paginated_list(list.into(), page, total)
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
    .spacing(SPACING_XXS);

    list_row(
        item_labels(volume.name.clone(), volume.driver.clone()),
        actions,
    )
}
