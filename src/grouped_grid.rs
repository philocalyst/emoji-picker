use emoji::{EmojiEntry, Group};
use gpui::{App, Context, IntoElement, ParentElement, Styled, Task, Window, div};
use gpui_component::{IndexPath, StyledExt, list::{ListDelegate, ListState}};

use crate::{core_row::EmojiRow, utilities::{grouped_emojis, search_emojis}};

pub(crate) struct GroupedEmojis {
	pub(crate) group:  Group,
	pub(crate) emojis: Vec<&'static EmojiEntry>,
}

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

	fn update_search(&mut self, query: &str) {
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

impl ListDelegate for EmojiListDelegate {
	type Item = EmojiRow;

	fn sections_count(&self, _: &App) -> usize { self.emoji_legions.len() }

	/// Get the total amount of items (emojis)
	fn items_count(&self, section: usize, _: &App) -> usize {
		let emoji_count = self
			.emoji_legions
			.get(section)
			.expect("Section numbers are generated at the same time as grouping creations")
			.emojis
			.len();

		(emoji_count + self.emojis_per_row - 1) / self.emojis_per_row
	}

	fn render_section_header(
		&self,
		section: usize,
		_: &mut Window,
		_: &mut App,
	) -> Option<impl IntoElement> {
		// Draw the current sections name as a psuedo-header
		self
			.emoji_legions
			.get(section)
			.map(|grouped| div().text_sm().font_semibold().child(format!("{:?}", grouped.group)))
	}

	/// Generate the relevant emoji for an index, as a struct to interpret
	fn render_item(&self, ix: IndexPath, _: &mut Window, _: &mut App) -> Option<Self::Item> {
		let section_emojis = &self.emoji_legions.get(ix.section)?.emojis;
		let start_idx = ix.row * self.emojis_per_row;
		let end_idx = (start_idx + self.emojis_per_row).min(section_emojis.len());

		if start_idx >= section_emojis.len() {
			return None;
		}

		let row_emojis = section_emojis[start_idx..end_idx].to_vec();

		Some(EmojiRow { emojis: row_emojis, selected: self.selected_index == Some(ix) })
	}

	/// Set the index and notify the context
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
		println!("Confirmed with secondary: {}", secondary);
	}

	/// Search, notify the context, and create a task to consume for the search
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
