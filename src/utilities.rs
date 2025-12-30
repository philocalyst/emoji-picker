use emoji::{EmojiEntry, lookup_by_glyph::iter_emoji};
use gpui::Pixels;

use crate::{SEARCHER, grouped_grid::GroupedEmojis};

/// Calculates the number of emojis that fit per row based on container width
pub(crate) fn calculate_emojis_per_row(container_width: f64, rem_size: Pixels) -> usize {
	let rem = rem_size.to_f64();

	// Outer padding for the list (left and right), now 1rem each side from
	// picker.rs
	let outer_padding = rem * 2.0;

	// Effective width available for all emoji items.
	let effective_width = container_width - outer_padding;

	// Each emoji item has padding from p_1(). Assuming p_1 is 0.25rem on each side.
	let inner_padding_per_emoji = rem * 0.5;

	// Ideal size of an emoji glyph. From previous implementation.
	let base_emoji_glyph_size = rem * 1.5;

	// Total width of one emoji item including its own padding.
	let total_width_per_emoji = base_emoji_glyph_size + inner_padding_per_emoji;

	let ideal_emojis_per_row = (effective_width / total_width_per_emoji).ceil() as usize;

	const MIN_EMOJIS_PER_ROW: usize = 8;
	const MAX_EMOJIS_PER_ROW: usize = 20;

	ideal_emojis_per_row.clamp(MIN_EMOJIS_PER_ROW, MAX_EMOJIS_PER_ROW)
}

/// Generate the grouped emoji vector that the application is based upon.
pub(crate) fn grouped_emojis() -> Vec<GroupedEmojis> {
	emoji::Group::iter().fold(Vec::from(vec![]), |mut all: Vec<GroupedEmojis>, current_group| {
		let group_emojis: Vec<&'static EmojiEntry> =
			iter_emoji().filter(|e| e.emoji().group == current_group).collect();

		all.push(GroupedEmojis { group: current_group, emojis: group_emojis });

		all
	})
}

/// Searches for emojis based on the provided text query
pub(crate) fn search_emojis(text: &str) -> Vec<&'static EmojiEntry> {
	match text {
		"" => iter_emoji().collect(),
		_ => {
			let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
			matcher.search_best_matching_emojis(text, Some(1000)).unwrap()
		}
	}
}
