use emoji::Emoji;
use gpui::{
	App, InteractiveElement, IntoElement, ParentElement, Styled, blue, div, hsla, prelude::*,
	transparent_black,
};
use gpui_component::{ActiveTheme, h_flex};

use crate::insert_emoji;

/// Renders the list of skin tone variants for a selected emoji
pub(crate) fn element(emoji: &Emoji, cx: &mut App) -> impl IntoElement {
	let variants = emoji.skin_tones.unwrap();

	h_flex()
		.gap_2()
		.p_0()
		.bg(blue())
		.children(variants.into_iter().map(|variant| {
			div()
				.child(variant.glyph)
				.text_size(gpui::px(24.))
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
