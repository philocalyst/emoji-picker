//! Renders the skin tone variant overlay row.

use std::{thread, time::Duration};

use emoji::Emoji;
use gpui::{
	App, InteractiveElement, IntoElement, ParentElement, Pixels, Render, RenderOnce,
	StatefulInteractiveElement, Styled, div, hsla,
};
use gpui_component::{gray_300, h_flex};

use crate::{components::variants::types::Variants, insert::insert_emoji};

impl RenderOnce for Variants {
	fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
		h_flex()
			.gap_2()
			.bg(gray_300())
			.children(self.available_emoji.clone().into_iter().map(|variant| {
				div()
					.child(variant.glyph)
					.text_size(self.font_size)
					.cursor_pointer()
					.id("hi")
					.hover(|s| s.bg(hsla(0., 0., 0., 0.1)))
					.on_click(move |_, _, cx: &mut App| {
						insert_emoji(variant.glyph, cx);
						thread::sleep(Duration::from_millis(100));
						cx.quit();
					})
			}))
			.into_any_element()
	}
}
