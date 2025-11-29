use emoji::{EmojiEntry, Group};
use gpui::{App, Context, IntoElement, ParentElement, RenderOnce, Styled, Task, Window, div};
use gpui_component::{IndexPath, Selectable, StyledExt, h_flex, list::{ListDelegate, ListState}};

use crate::{emojis_in_a_row::Emoji, utilities::search_emojis};

pub(crate) struct GroupedEmojis {
	pub(crate) group:  Group,
	pub(crate) emojis: Vec<&'static EmojiEntry>,
}

pub(crate) struct EmojiListDelegate {
	pub(crate) grouped_emojis: Vec<GroupedEmojis>,
	pub(crate) emojis_per_row: usize,
	pub(crate) selected_index: Option<IndexPath>,
	pub(crate) query:          String,
}

impl EmojiListDelegate {
	pub(crate) fn new(emojis_per_row: usize) -> Self {
		let mut grouped = Vec::new();

		for group in emoji::Group::iter() {
			let group_emojis: Vec<&'static EmojiEntry> =
				emoji::lookup_by_glyph::iter_emoji().filter(|e| e.emoji().group == group).collect();

			if !group_emojis.is_empty() {
				grouped.push(GroupedEmojis { group, emojis: group_emojis });
			}
		}

		Self { grouped_emojis: grouped, emojis_per_row, selected_index: None, query: String::new() }
	}

	fn update_search(&mut self, query: &str) {
		self.query = query.to_string();

		self.grouped_emojis.clear();

		if query.is_empty() {
			for group in emoji::Group::iter() {
				let group_emojis: Vec<&'static EmojiEntry> =
					emoji::lookup_by_glyph::iter_emoji().filter(|e| e.emoji().group == group).collect();

				if !group_emojis.is_empty() {
					self.grouped_emojis.push(GroupedEmojis { group, emojis: group_emojis });
				}
			}
		} else {
			let filtered = search_emojis(query);

			for group in emoji::Group::iter() {
				let group_emojis: Vec<&'static EmojiEntry> =
					filtered.iter().filter(|e| e.emoji().group == group).copied().collect();

				if !group_emojis.is_empty() {
					self.grouped_emojis.push(GroupedEmojis { group, emojis: group_emojis });
				}
			}
		}
	}
}

impl ListDelegate for EmojiListDelegate {
	type Item = Emoji;

	fn sections_count(&self, _: &App) -> usize { self.grouped_emojis.len() }

	fn items_count(&self, section: usize, _: &App) -> usize {
		if section >= self.grouped_emojis.len() {
			return 0;
		}
		let emoji_count = self.grouped_emojis[section].emojis.len();
		(emoji_count + self.emojis_per_row - 1) / self.emojis_per_row
	}

	fn render_section_header(
		&self,
		section: usize,
		_: &mut Window,
		_: &mut App,
	) -> Option<impl IntoElement> {
		self.grouped_emojis.get(section).map(|grouped| {
			div().px_2().py_1().text_sm().font_semibold().child(format!("{:?}", grouped.group))
		})
	}

	fn render_item(&self, ix: IndexPath, _: &mut Window, _: &mut App) -> Option<Self::Item> {
		let section_emojis = &self.grouped_emojis.get(ix.section)?.emojis;
		let row_starting = ix.row * self.emojis_per_row;

		if row_starting >= section_emojis.len() {
			return None;
		}

		let current_emoji = row_starting + ix.column;

		Some(Emoji {
			emoji:    section_emojis[current_emoji],
			selected: self.selected_index == Some(ix),
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
		println!("Confirmed with secondary: {}", secondary);
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
