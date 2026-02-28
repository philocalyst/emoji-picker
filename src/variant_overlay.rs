use emoji::Emoji;
use gpui::{
	App, InteractiveElement, IntoElement, ParentElement, Pixels, Styled, blue, div, hsla, prelude::*,
	transparent_black,
};
use gpui_component::{ActiveTheme, gray, gray_300, h_flex};

use crate::insert_emoji;

/// Renders the list of skin tone variants for a selected emoji
pub(crate) fn element(emoji: &Emoji, font_size: Pixels) -> impl IntoElement {
	let variants = emoji.skin_tones.unwrap();

	h_flex()
		.gap_2()
		.bg(gray_300())
		.children(variants.into_iter().map(|variant| {
			div()
				.child(variant.glyph)
				.text_size(font_size)
				.cursor_pointer()
				.id("hi")
				.hover(|s| s.bg(hsla(0., 0., 0., 0.1)))
				.on_click(move |_, _, cx: &mut App| {
					insert_emoji(variant.glyph);
					// cx.window().shutdown();
					cx.shutdown();
				})
		}))
		.into_any_element()
}
