use emoji::{EmojiEntry, Group};
use gpui::{App, Context, IntoElement, ParentElement, Styled, Task, Window, div};
use gpui_component::{IndexPath, StyledExt, list::{ListDelegate, ListState}};

use crate::{core_row::EmojiRow, listgistics::EmojiListDelegate};

pub(crate) struct GroupedEmojis {
	pub(crate) group:  Group,
	pub(crate) emojis: Vec<&'static EmojiEntry>,
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

	// TODO: Don't render when searching
	fn render_section_header(
		&self,
		section: usize,
		_: &mut Window,
		_: &mut App,
	) -> Option<impl IntoElement> {
		// Don't show when searching, as the limited results make the headers feel
		// cramped
		if !self.query.is_empty() {
			return None;
		}

		// Draw the current sections name as a psuedo-header
		self.emoji_legions.get(section).map(|grouped| {
			div().underline().text_lg().font_semibold().pb_2().pt_2().child(grouped.group.to_string())
		})
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

		Some(EmojiRow {
			emojis:    row_emojis,
			selected:  self.selected_index == Some(ix),
			font_size: self.emoji_size,
		})
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
