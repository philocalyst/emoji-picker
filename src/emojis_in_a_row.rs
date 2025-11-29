use emoji::EmojiEntry;
use gpui::{App, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder};
use gpui_component::{ActiveTheme, Selectable, h_flex};

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
	fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
		h_flex().gap_2().children(self.emojis.iter().map(|emoji| {
			div()
				.id(emoji.emoji().glyph)
				.when(self.selected, |div| div.bg(cx.theme().accent.opacity(0.5)))
				.on_click(|click_event, window, app| {
					dbg!("hi");
				})
				.cursor_pointer()
				.child(emoji.emoji().glyph)
		}))
	}
}
