use emoji::{Emoji, Group};
use gpui::{App, Context, IntoElement, ParentElement, Styled, Task, Window, div};
use gpui_component::{
	IndexPath, StyledExt,
	list::{ListDelegate, ListState},
};

use crate::{core_row::EmojiRow, listgistics::EmojiListDelegate};

pub(crate) struct GroupedEmojis {
	pub(crate) group: Group,
	pub(crate) emojis: Vec<&'static Emoji>,
}

impl ListDelegate for EmojiListDelegate {
	type Item = EmojiRow;

	fn sections_count(&self, _: &App) -> usize {
		self.emoji_legions.len()
	}

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
		&mut self,
		section: usize,
		_: &mut Window,
		_: &mut Context<'_, ListState<Self>>,
	) -> Option<impl IntoElement> {
		// Don't show when searching, as the limited results make the headers feel
		// cramped
		if !self.query.is_empty() {
			return None;
		}

		// Draw the current sections name as a psuedo-header
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

	/// Generate the relevant emoji for an index, as a struct to interpret
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
		let selected_col = if is_selected_row { self.selected_index.map(|sel| sel.column) } else { None };

		Some(EmojiRow {
			emojis: row_emojis,
			selected: is_selected_row,
			selected_column: selected_col,
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
