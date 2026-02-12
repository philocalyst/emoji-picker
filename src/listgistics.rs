
use gpui::Pixels;
use gpui_component::IndexPath;

use crate::{grouped_grid::GroupedEmojis, utilities::{grouped_emojis, search_emojis}};

pub(crate) struct EmojiListDelegate {
	pub(crate) emoji_legions:  Vec<GroupedEmojis>,
	pub(crate) emojis_per_row: usize,
	pub(crate) selected_index: Option<IndexPath>,
	pub(crate) query:          String,
	pub(crate) emoji_size:     Pixels,
}

impl EmojiListDelegate {
	pub(crate) fn new(emojis_per_row: usize, emoji_size: Pixels) -> Self {
		Self {
			emoji_legions: grouped_emojis(),
			emoji_size,
			emojis_per_row,
			selected_index: None,
			query: String::new(),
		}
	}

	pub(crate) fn update_search(&mut self, query: &str) {
		self.query = query.to_string();

		self.emoji_legions.clear();

		if query.is_empty() {
			self.emoji_legions = grouped_emojis();
		} else {
			let filtered = search_emojis(query);

			self.emoji_legions.push(GroupedEmojis { group: emoji::Group::PeopleBody, emojis: filtered });
		}
	}
}
