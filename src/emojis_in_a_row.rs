use emoji::EmojiEntry;
use gpui::{App, Edges, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder};
use gpui_component::{ActiveTheme, Selectable, StyledExt, h_flex};

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
				.hover(|div| div.bg(cx.theme().accent.opacity(1.0)))
				.on_click(|click_event, window, app| {
					espanso_inject::get_injector(espanso_inject::InjectorCreationOptions::default())
						.unwrap()
						.send_string("hi", espanso_inject::InjectionOptions::default());
				})
				.cursor_pointer()
				.child(emoji.emoji().glyph)
		}))
	}
}
