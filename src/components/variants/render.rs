//! Renders the skin tone variant overlay row.

use gpui::{
	App, InteractiveElement, IntoElement, ParentElement, RenderOnce,
	StatefulInteractiveElement, Styled, div, hsla,
};
use gpui_component::{gray_300, h_flex};

use crate::{components::variants::types::Variants, insert::close_and_insert};

impl RenderOnce for Variants {
	fn render(self, _window: &mut gpui::Window, _cx: &mut App) -> impl IntoElement {
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
						close_and_insert(variant.glyph, cx);
					})
			}))
			.into_any_element()
	}
}
