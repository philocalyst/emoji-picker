use emoji::EmojiEntry;
use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div};
use gpui_component::{Selectable, h_flex};

#[derive(IntoElement)]
pub(crate) struct EmojiRow {
	pub(crate) emojis:   Vec<&'static EmojiEntry>,
	pub(crate) selected: bool,
}

impl Selectable for EmojiRow {
	fn selected(mut self, selected: bool) -> Self {
		self.selected = selected;
		self
	}

	fn is_selected(&self) -> bool { self.selected }
}

impl RenderOnce for EmojiRow {
	fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
		h_flex()
			.gap_2()
			.children(self.emojis.iter().map(|emoji| div().cursor_pointer().child(emoji.emoji().glyph)))
	}
}
