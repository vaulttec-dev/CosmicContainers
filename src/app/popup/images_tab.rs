use cosmic::Element;
use cosmic::widget;
use linux_pop_extension::ImageSummary;
use linux_pop_extension::action_icons;

use super::layout::SPACING_NONE;
use super::layout::SPACING_XXS;
use super::widgets::{
    action_svg_button, empty_state, item_labels, list_row, remove_confirm_row, scrollable_list,
};
use super::{App, Message};
use crate::app::PendingRemove;

pub(super) fn images_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.image_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No images");
    }

    let mut list = widget::column![].spacing(SPACING_NONE);
    for image in &state.image_list {
        list = list.push(image_row(image, &state.pending_remove));
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
    .spacing(SPACING_XXS);

    list_row(
        item_labels(
            image.reference.clone(),
            format!("{} · {}", image.size, image.id),
        ),
        actions,
    )
}
