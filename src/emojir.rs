use std::rc::Rc;

use emoji::Emoji;
use gpui::Div;
use gpui::Entity;
use gpui::InteractiveElement;
use gpui::IntoElement;
use gpui::ParentElement;
use gpui::Pixels;
use gpui::Size;
use gpui::StatefulInteractiveElement;
use gpui::Styled;
use gpui::div;
use gpui::rems;
use gpui::white;
use gpui_component::VirtualListScrollHandle;
use gpui_component::v_virtual_list;

use crate::picker::Picker;

/// Renders a single emoji button
pub(crate) fn render_button(emoji_idx: usize, emoji: &Emoji) -> impl IntoElement {
    div()
        .id(emoji_idx)
        .child(emoji.glyph)
        .cursor_pointer()
        .on_click({
            let moji = emoji.to_owned();
            move |_e, _w, _cx| println!("{moji:?}")
        })
}

/// Renders a row of emojis for the virtual list
pub(crate) fn render_row<'a>(start_idx: usize, end_idx: usize, emojis: &'a [&'a Emoji]) -> Div {
    div()
        .flex()
        .flex_row()
        .children((start_idx..end_idx).map(|emoji_idx| {
            let moji = emojis[emoji_idx];
            render_button(emoji_idx, moji)
        }))
}

/// Renders the emoji grid with virtual scrolling
pub(crate) fn render_grid(
    entity: Entity<Picker>,
    emojis: Vec<&'static Emoji>,
    emojis_per_row: usize,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    emoji_text_size: f32,
    scroll_handle: &VirtualListScrollHandle,
) -> impl IntoElement {
    v_virtual_list(
        entity,
        "emojis",
        row_sizes,
        move |_container: &mut Picker, range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|row_idx| {
                    let start_idx = row_idx * emojis_per_row;
                    let end_idx = (start_idx + emojis_per_row).min(emojis.len()) - 1;
                    render_row(start_idx, end_idx, &emojis)
                })
                .collect()
        },
    )
    .text_size(rems(emoji_text_size))
    .track_scroll(scroll_handle)
    .h_full()
}
