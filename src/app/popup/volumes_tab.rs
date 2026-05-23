use cosmic::Element;
use cosmic::widget;
use linux_pop_extension::VolumeSummary;
use linux_pop_extension::action_icons;

use super::layout::SPACING_NONE;
use super::layout::SPACING_XXS;
use super::widgets::{
    action_svg_button, empty_state, item_labels, list_row, remove_confirm_row, scrollable_list,
};
use super::{App, Message};
use crate::app::PendingRemove;

pub(super) fn volumes_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.volume_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("Немає томів");
    }

    let mut list = widget::column![].spacing(SPACING_NONE);
    for volume in &state.volume_list {
        list = list.push(volume_row(volume, &state.pending_remove));
    }
    scrollable_list(list.into())
}

fn volume_row<'a>(
    volume: &'a VolumeSummary,
    pending_remove: &Option<PendingRemove>,
) -> Element<'a, Message> {
    let name = volume.name.clone();
    if matches!(pending_remove, Some(PendingRemove::Volume(n)) if n == &name) {
        return remove_confirm_row(format!("Видалити том {name}?"));
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
