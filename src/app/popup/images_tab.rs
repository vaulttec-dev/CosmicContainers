use cosmic::Element;
use cosmic::widget;
use cosmic_containers::ImageSummary;
use cosmic_containers::action_icons;
use cosmic_containers::{LIST_PAGE_SIZE, clamp_page, page_range, total_pages};

use super::layout::SPACING_NONE;
use super::layout::SPACING_XXS;
use super::widgets::{
    action_svg_button, empty_state, item_labels, list_row, paginated_list, remove_confirm_row,
};
use super::{App, Message};
use crate::app::PendingRemove;

pub(super) fn images_tab<'a>(state: &'a App) -> Element<'a, Message> {
    if state.image_list.is_empty() && !state.loading && state.error.is_none() {
        return empty_state("No images");
    }

    let total = total_pages(state.image_list.len(), LIST_PAGE_SIZE);
    let page = clamp_page(state.tab_pages.images, total);
    let range = page_range(page, LIST_PAGE_SIZE, state.image_list.len());

    let mut list = widget::column![].spacing(SPACING_NONE);
    for image in &state.image_list[range] {
        list = list.push(image_row(image, &state.pending_remove));
    }
    paginated_list(list.into(), page, total)
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
