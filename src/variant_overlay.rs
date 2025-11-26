use emoji::Emoji;
use gpui::IntoElement;
use gpui::ParentElement;
use gpui::Styled;
use gpui::div;
use gpui::white;

use crate::utils::generate_skin_tone_variants;

/// Renders the overlay showing skin tone variants for a selected emoji
pub(crate) fn render(emoji: &Emoji) -> impl IntoElement {
    if let Some(variants) = generate_skin_tone_variants(emoji.glyph) {
        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .p_4()
                    .rounded_md()
                    .shadow_lg()
                    .flex()
                    .flex_row()
                    .bg(white())
                    .gap_2()
                    .children(variants.into_iter().map(|variant| div().child(variant))),
            )
    } else {
        div()
    }
}
