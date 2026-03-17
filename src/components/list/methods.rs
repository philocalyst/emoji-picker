//! List delegate construction, search, grid navigation, and ListDelegate trait implementation.

use emoji::{Emoji, lookup_by_glyph::ALL_EMOJI};
use gpui::{App, Context, FocusHandle, IntoElement, ParentElement, Pixels, Styled, Task, Window, div};
use gpui_component::{IndexPath, StyledExt, list::{ListDelegate, ListState}};
use tracing::debug;

use crate::components::list::row::types::EmojiRow;
use super::types::{EmojiListDelegate, GroupedEmojis};

fn grouped_emojis() -> Vec<GroupedEmojis> {
	emoji::Group::iter().fold(Vec::new(), |mut all, current_group| {
		let group_emojis: Vec<&'static Emoji> =
			ALL_EMOJI.iter().filter(|e| e.group == current_group).cloned().collect();
		all.push(GroupedEmojis { group: current_group, emojis: group_emojis });
		all
	})
}

fn search_emojis(text: &str) -> Vec<&'static Emoji> {
	let matcher: &'static emoji_search::EmojiSearcher = &*super::SEARCHER;
	matcher.search_best_matching_emojis(text, Some(100)).unwrap()
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
			debug!(query = %query, results = filtered.len(), "search updated");
			self.emoji_legions.push(GroupedEmojis { group: emoji::Group::PeopleBody, emojis: filtered });
		}
	}

	pub(crate) fn move_right(&mut self) {
		debug!("move right: {:?}", self.selected_index);
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
	}

	pub(crate) fn move_left(&mut self) {
		debug!("move left: {:?}", self.selected_index);
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
	}

	pub(crate) fn move_down(&mut self) {
		debug!("move down: {:?}", self.selected_index);
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
	}

	pub(crate) fn move_up(&mut self) {
		debug!("move up: {:?}", self.selected_index);
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
	}
}

impl ListDelegate for EmojiListDelegate {
	type Item = EmojiRow;

	fn sections_count(&self, _: &App) -> usize {
		self.emoji_legions.len()
	}

	fn items_count(&self, section: usize, _: &App) -> usize {
		let emoji_count = self
			.emoji_legions
			.get(section)
			.expect("section index out of bounds")
			.emojis
			.len();
		(emoji_count + self.emojis_per_row - 1) / self.emojis_per_row
	}

	fn render_section_header(
		&mut self,
		section: usize,
		_: &mut Window,
		_: &mut Context<'_, ListState<Self>>,
	) -> Option<impl IntoElement> {
		if !self.query.is_empty() {
			return None;
		}

		self.emoji_legions.get(section).map(|grouped| {
			let label = grouped
				.group
				.to_string()
				.replace('-', " & ")
				.to_uppercase()
				.chars()
				.flat_map(|c| if c == ' ' { vec![' ', ' '] } else { vec![c, ' '] })
				.collect::<String>()
				.trim_end()
				.to_string();
			div().underline().text_lg().font_semibold().pb_2().pt_2().child(label)
		})
	}

	fn render_item(
		&mut self,
		ix: IndexPath,
		_: &mut Window,
		_: &mut Context<ListState<Self>>,
	) -> Option<Self::Item> {
		let section_emojis = &self.emoji_legions.get(ix.section)?.emojis;
		let start_idx = ix.row * self.emojis_per_row;
		let end_idx = (start_idx + self.emojis_per_row).min(section_emojis.len());

		if start_idx >= section_emojis.len() {
			return None;
		}

		let row_emojis = section_emojis[start_idx..end_idx].to_vec();

		let is_selected_row = self
			.selected_index
			.map(|sel| sel.section == ix.section && sel.row == ix.row)
			.unwrap_or(false);
		let selected_col =
			if is_selected_row { self.selected_index.map(|sel| sel.column) } else { None };

		Some(EmojiRow {
			emojis: row_emojis,
			selected: is_selected_row,
			contains_selection: is_selected_row,
			selected_column: selected_col,
			font_size: self.emoji_size,
			body_focus_handle: self.body_focus_handle.clone(),
		})
	}

	fn set_selected_index(
		&mut self,
		ix: Option<IndexPath>,
		_: &mut Window,
		cx: &mut Context<ListState<Self>>,
	) {
		self.selected_index = ix;
		cx.notify();
	}

	fn confirm(&mut self, secondary: bool, _: &mut Window, _: &mut Context<ListState<Self>>) {
		debug!(secondary, "list confirm");
	}

	fn perform_search(
		&mut self,
		query: &str,
		_: &mut Window,
		cx: &mut Context<ListState<Self>>,
	) -> Task<()> {
		self.update_search(query);
		cx.notify();
		Task::ready(())
	}
}
