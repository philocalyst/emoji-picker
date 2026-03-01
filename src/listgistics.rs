use gpui::{FocusHandle, Pixels};
use gpui_component::IndexPath;
use tracing::info;

use crate::{grouped_grid::GroupedEmojis, utilities::{grouped_emojis, search_emojis}};

pub(crate) struct EmojiListDelegate {
	pub(crate) emoji_legions:     Vec<GroupedEmojis>,
	pub(crate) emojis_per_row:    usize,
	pub(crate) selected_index:    Option<IndexPath>,
	pub(crate) query:             String,
	/// Body focus handle
	pub(crate) body_focus_handle: FocusHandle,
	pub(crate) emoji_size:        Pixels,
}

impl EmojiListDelegate {
	pub(crate) fn new(
		emojis_per_row: usize,
		emoji_size: Pixels,
		body_focus_handle: FocusHandle,
	) -> Self {
		Self {
			body_focus_handle,
			emoji_legions: grouped_emojis(),
			emoji_size,
			emojis_per_row,
			selected_index: Some(IndexPath { section: 0, row: 0, column: 0 }),
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

	pub(crate) fn move_right(&mut self) {
		info!("Move Right: {:?}", self.selected_index);
		if let Some(mut ix) = self.selected_index {
			if let Some(section) = self.emoji_legions.get(ix.section) {
				let section_len = section.emojis.len();
				let flat_index = ix.row * self.emojis_per_row + ix.column;

				if flat_index + 1 < section_len {
					if ix.column + 1 < self.emojis_per_row {
						ix.column += 1;
					} else {
						ix.row += 1;
						ix.column = 0;
					}
					self.selected_index = Some(ix);
				} else if ix.section + 1 < self.emoji_legions.len() {
					self.selected_index = Some(IndexPath { section: ix.section + 1, row: 0, column: 0 });
				}
			}
		} else if !self.emoji_legions.is_empty() {
			self.selected_index = Some(IndexPath { section: 0, row: 0, column: 0 });
		}
		info!("Moved Right to: {:?}", self.selected_index);
	}

	pub(crate) fn move_left(&mut self) {
		info!("Move Left: {:?}", self.selected_index);
		if let Some(mut ix) = self.selected_index {
			if ix.column > 0 {
				ix.column -= 1;
				self.selected_index = Some(ix);
			} else if ix.row > 0 {
				ix.row -= 1;
				ix.column = self.emojis_per_row - 1;
				self.selected_index = Some(ix);
			} else if ix.section > 0 {
				let prev_section_idx = ix.section - 1;
				if let Some(prev_section) = self.emoji_legions.get(prev_section_idx) {
					let count = prev_section.emojis.len();
					if count > 0 {
						let last_idx = count - 1;
						let last_row = last_idx / self.emojis_per_row;
						let last_col = last_idx % self.emojis_per_row;
						self.selected_index =
							Some(IndexPath { section: prev_section_idx, row: last_row, column: last_col });
					}
				}
			}
		} else if !self.emoji_legions.is_empty() {
			self.selected_index = Some(IndexPath { section: 0, row: 0, column: 0 });
		}
		info!("Moved Left to: {:?}", self.selected_index);
	}

	pub(crate) fn move_down(&mut self) {
		info!("Move Down: {:?}", self.selected_index);
		if let Some(mut ix) = self.selected_index {
			if let Some(section) = self.emoji_legions.get(ix.section) {
				let section_len = section.emojis.len();
				let next_row_start = (ix.row + 1) * self.emojis_per_row;

				if next_row_start < section_len {
					ix.row += 1;
					let items_in_next_row = (section_len - next_row_start).min(self.emojis_per_row);
					if ix.column >= items_in_next_row {
						ix.column = items_in_next_row - 1;
					}
					self.selected_index = Some(ix);
				} else if ix.section + 1 < self.emoji_legions.len() {
					let next_sec_len = self.emoji_legions[ix.section + 1].emojis.len();
					if next_sec_len > 0 {
						let next_col = ix.column.min(next_sec_len - 1).min(self.emojis_per_row - 1);
						self.selected_index =
							Some(IndexPath { section: ix.section + 1, row: 0, column: next_col });
					}
				}
			}
		} else if !self.emoji_legions.is_empty() {
			self.selected_index = Some(IndexPath { section: 0, row: 0, column: 0 });
		}
		info!("Moved Down to: {:?}", self.selected_index);
	}

	pub(crate) fn move_up(&mut self) {
		info!("Move Up: {:?}", self.selected_index);
		if let Some(mut ix) = self.selected_index {
			if ix.row > 0 {
				ix.row -= 1;
				self.selected_index = Some(ix);
			} else if ix.section > 0 {
				let prev_section_idx = ix.section - 1;
				if let Some(prev_section) = self.emoji_legions.get(prev_section_idx) {
					let count = prev_section.emojis.len();
					if count > 0 {
						let last_row = (count - 1) / self.emojis_per_row;
						let items_in_last_row = count - (last_row * self.emojis_per_row);
						let col = ix.column.min(items_in_last_row - 1);
						self.selected_index =
							Some(IndexPath { section: prev_section_idx, row: last_row, column: col });
					}
				}
			}
		} else if !self.emoji_legions.is_empty() {
			self.selected_index = Some(IndexPath { section: 0, row: 0, column: 0 });
		}
		info!("Moved Up to: {:?}", self.selected_index);
	}
}
