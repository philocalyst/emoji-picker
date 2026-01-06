use emoji::EmojiEntry;
use gpui::{App, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, px};
use gpui_component::StyledExt;
pub(crate) use gpui_component::{ActiveTheme, Selectable, h_flex};
use nonempty::NonEmpty;

use crate::{SelectedEmoji, insert_emoji};

#[derive(IntoElement)]
pub(crate) struct EmojiRow {
	/// The Emoji contained by the row
	pub(crate) emojis: Vec<&'static EmojiEntry>,

	/// Whether the row has been selected
	pub(crate) selected: bool,

	/// The size of each emoji
	pub(crate) font_size: gpui::Pixels,
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
		// Cache the theme color here so we don't capture cx in .hover()
		let hover_bg = cx.theme().accent;

		h_flex().gap_2().children(self.emojis.into_iter().map(move |emoji| {
			let emoji_data = emoji.emoji().clone();
			let other_emoji = emoji.emoji().clone();

			div()
				.text_size(self.font_size)
				.id(emoji_data.glyph)
				.hover(move |div| div.bg(hover_bg))
				.on_click(move |_click_event, _window, cx| {
					insert_emoji(emoji.emoji().glyph);

					cx.shutdown();
				})
				.corner_radii(gpui::Corners::all(px(5f32)))
				.cursor_pointer()
				.child(other_emoji.glyph)
		}))
	}
}
