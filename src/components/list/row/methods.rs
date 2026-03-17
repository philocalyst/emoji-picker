//! Trait implementations for EmojiWrapper.

use gpui::{App, IntoElement, RenderOnce, Window};
use gpui_component::Selectable;

use super::types::EmojiWrapper;

impl Selectable for EmojiWrapper {
	fn selected(mut self, selected: bool) -> Self {
		self.selected = selected;
		self
	}

	fn is_selected(&self) -> bool { self.selected }
}

impl IntoElement for EmojiWrapper {
	type Element = gpui::AnyElement;

	fn into_element(self) -> Self::Element { self.content }
}

impl RenderOnce for EmojiWrapper {
	fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement { self.content }
}
