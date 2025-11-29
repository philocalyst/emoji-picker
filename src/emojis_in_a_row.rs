use emoji::EmojiEntry;
use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div};
use gpui_component::{Selectable, gray_200, h_flex};

#[derive(IntoElement)]
pub(crate) struct Emoji {
	pub(crate) emoji:    &'static EmojiEntry,
	pub(crate) selected: bool,
}

impl Selectable for Emoji {
	fn selected(mut self, selected: bool) -> Self {
		self.selected = selected;
		self
	}

	fn is_selected(&self) -> bool { self.selected }
}

impl RenderOnce for Emoji {
	fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
		let div = match self.selected {
			true => div().bg(gray_200()),
			false => div(),
		};

		h_flex().gap_2().child(div.cursor_pointer().child(self.emoji.emoji().glyph))
	}
}
