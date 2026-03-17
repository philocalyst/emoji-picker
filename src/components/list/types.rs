//! Data types for the emoji list delegate and grouped emoji collections.

use emoji::{Emoji, Group};
use gpui::{FocusHandle, Pixels};
use gpui_component::IndexPath;

pub(crate) struct GroupedEmojis {
	pub(crate) group: Group,
	pub(crate) emojis: Vec<&'static Emoji>,
}

pub(crate) struct EmojiListDelegate {
	pub(crate) emoji_legions: Vec<GroupedEmojis>,
	pub(crate) emojis_per_row: usize,
	pub(crate) selected_index: Option<IndexPath>,
	pub(crate) query: String,
	pub(crate) body_focus_handle: FocusHandle,
	pub(crate) emoji_size: Pixels,
}
