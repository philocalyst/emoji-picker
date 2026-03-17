//! Types for individual emoji rows and their wrapper elements.

use emoji::Emoji;
use gpui::{AnyElement, FocusHandle, IntoElement};
use gpui_component::Selectable;

#[derive(IntoElement)]
pub(crate) struct EmojiRow {
	pub(crate) emojis:             Vec<&'static Emoji>,
	pub(crate) body_focus_handle:  FocusHandle,
	pub(crate) selected:           bool,
	pub(crate) contains_selection: bool,
	pub(crate) selected_column:    Option<usize>,
	pub(crate) font_size:          gpui::Pixels,
}

impl Selectable for EmojiRow {
	fn selected(mut self, selected: bool) -> Self {
		self.selected = selected;
		self
	}

	fn is_selected(&self) -> bool { self.selected }
}

pub(crate) struct EmojiWrapper {
	pub(crate) content:  AnyElement,
	pub(crate) selected: bool,
}
