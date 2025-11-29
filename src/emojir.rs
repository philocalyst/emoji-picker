use emoji::{Emoji, EmojiEntry, Group};
use gpui::{App, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, Styled, Subscription, Task, Window, div, prelude::FluentBuilder};
use gpui_component::{IndexPath, Selectable, StyledExt, h_flex, input::InputState, list::{List, ListDelegate, ListEvent, ListState}, v_flex};

use crate::{input, utils::{calculate_emojis_per_row, search_emojis}};

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

#[derive(IntoElement)]
pub(crate) struct EmojiRow {
	emojis:   Vec<&'static EmojiEntry>,
	selected: bool,
}

impl Selectable for EmojiRow {
	fn selected(mut self, selected: bool) -> Self {
		self.selected = selected;
		self
	}

	fn is_selected(&self) -> bool { self.selected }
}

impl RenderOnce for EmojiRow {
	fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
		h_flex()
			.gap_2()
			.children(self.emojis.iter().map(|emoji| div().cursor_pointer().child(emoji.emoji().glyph)))
	}
}

impl ListDelegate for EmojiListDelegate {
	type Item = EmojiRow;

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
		let start_idx = ix.row * self.emojis_per_row;
		let end_idx = (start_idx + self.emojis_per_row).min(section_emojis.len());

		if start_idx >= section_emojis.len() {
			return None;
		}

		let row_emojis = section_emojis[start_idx..end_idx].to_vec();

		Some(EmojiRow { emojis: row_emojis, selected: self.selected_index == Some(ix) })
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
