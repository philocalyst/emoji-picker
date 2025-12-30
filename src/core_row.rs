use emoji::EmojiEntry;
use gpui::{App, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, px};
use gpui_component::StyledExt;
pub(crate) use gpui_component::{ActiveTheme, Selectable, h_flex};

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
		h_flex().gap_2().children(self.emojis.iter().map(|emoji| {
			div().text_size(self.font_size)
				.id(emoji.emoji().glyph) // ID is required for jump points
				.hover(|div| div.bg(cx.theme().accent.opacity(0.7))) // Bring out the background for hover contrast
				.on_click(|_click_event, _window, _app| {
					espanso_inject::get_injector(espanso_inject::InjectorCreationOptions::default())
						.unwrap()
						.send_string("hi", espanso_inject::InjectionOptions::default())
						.expect("Shouldn't fail, I trust Espanso");
				})
				.corner_radii(gpui::Corners::all(px(5f32)))
				.cursor_pointer()
				.child(emoji.emoji().glyph)
		}))
	}
}
