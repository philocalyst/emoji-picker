use emoji::EmojiEntry;
use gpui::{App, BoxShadow, Edges, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, hsla, px, red, transparent_black, transparent_white};
use gpui_component::StyledExt;
pub(crate) use gpui_component::{ActiveTheme, Selectable, h_flex};
use nonempty::NonEmpty;

use crate::{SelectedEmoji, ToneIndex, insert_emoji};

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

		let mut padding = Edges::all(px(10.));
		padding.right = px(0.);
		padding.left = px(0.);

		h_flex().paddings(padding).gap_2().children(self.emojis.into_iter().map(move |emoji| {
			let tone_index = cx.global::<ToneIndex>();

			// Get the right tone
			let pure_emoji = match emoji {
				EmojiEntry::Standard(emoji) => emoji.glyph,
				EmojiEntry::Toned(toned_emoji) => {
					toned_emoji.tones.get(tone_index.0 as usize).unwrap_or(&toned_emoji.emoji).glyph
				}
			};

			div()
				.bg(transparent_black())
				.text_size(self.font_size)
				.id(pure_emoji)
				.shadow(vec![BoxShadow {
					color:         hsla(0.0, 0.0, 0.0, 0.15),
					offset:        gpui::point(gpui::px(0.), gpui::px(4.)), // Light from above
					blur_radius:   gpui::px(10.),
					spread_radius: gpui::px(1.),
				}])
				.hover(move |div| div.bg(hover_bg))
				.on_click(move |_click_event, _window, cx| {
					insert_emoji(emoji.emoji().glyph);

					cx.shutdown();
				})
				.corner_radii(gpui::Corners::all(px(5f32)))
				.cursor_pointer()
				.child(pure_emoji)
		}))
	}
}
