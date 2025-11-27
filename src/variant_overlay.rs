use emoji::{Emoji, EmojiEntry};
use gpui::{IntoElement, ParentElement, Styled, div, white};

/// Renders the overlay showing skin tone variants for a selected emoji
pub(crate) fn render(emoji: &EmojiEntry) -> impl IntoElement {
	if let Some(variants) = emoji.tones() {
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
					.children(variants.into_iter().map(|variant| div().child(variant.glyph))),
			)
	} else {
		div()
	}
}
