pub(crate) use emoji::EmojiEntry;
use gpui_component::IndexPath;

use crate::{grouped_grid::GroupedEmojis, utilities::{grouped_emojis, search_emojis}};

pub(crate) struct EmojiListDelegate {
	pub(crate) emoji_legions:  Vec<GroupedEmojis>,
	pub(crate) emojis_per_row: usize,
	pub(crate) selected_index: Option<IndexPath>,
	pub(crate) query:          String,
}

impl EmojiListDelegate {
	pub(crate) fn new(emojis_per_row: usize) -> Self {
		Self {
			emoji_legions: grouped_emojis(),
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

			for group in emoji::Group::iter() {
				let group_emojis: Vec<&'static EmojiEntry> =
					filtered.iter().filter(|e| e.emoji().group == group).copied().collect();

				if !group_emojis.is_empty() {
					self.emoji_legions.push(GroupedEmojis { group, emojis: group_emojis });
				}
			}
		}
	}
}
