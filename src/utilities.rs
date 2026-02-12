use emoji::{Emoji, lookup_by_glyph::ALL_EMOJI};
use gpui::Pixels;

use crate::{SEARCHER, grouped_grid::GroupedEmojis};

/// Calculates emoji sizing parameters to prevent overflow
pub(crate) struct EmojiSizing {
	pub emojis_per_row: usize,
	pub emoji_size:     Pixels,
	pub list_padding:   Pixels,
}

/// Calculates the number of emojis that fit per row based on container width
/// Returns both the count and the proper emoji size to prevent overflow
pub(crate) fn calculate_emoji_sizing(container_width: f64, rem_size: Pixels) -> EmojiSizing {
	let rem = rem_size.to_f64();

	const MIN_EMOJIS_PER_ROW: usize = 8;
	const MAX_EMOJIS_PER_ROW: usize = 20;

	// Target list padding as a ratio of emoji size (we'll calculate this
	// iteratively)
	let list_padding_ratio = 0.25;

	// Each emoji has p_2() padding which is 0.25rem on each side
	let emoji_padding_per_side = rem * 0.25;
	let total_emoji_padding = emoji_padding_per_side * 2.0;

	// Try different emoji counts to find the best fit
	let mut best_size = EmojiSizing {
		emojis_per_row: MIN_EMOJIS_PER_ROW,
		emoji_size:     Pixels::from(0.0),
		list_padding:   Pixels::from(0.0),
	};

	for emojis_per_row in MIN_EMOJIS_PER_ROW..=MAX_EMOJIS_PER_ROW {
		// Calculate what the emoji size would need to be for this count
		// Formula: container_width = (list_padding * 2) + (emojis_per_row * (emoji_size
		// + total_emoji_padding)) Where list_padding = emoji_size *
		// list_padding_ratio

		let denominator = (2.0 * list_padding_ratio) + emojis_per_row as f64;
		let numerator = container_width - (emojis_per_row as f64 * total_emoji_padding);
		let emoji_size = numerator / denominator;

		// We want emoji size to be reasonable (between 1rem and 3rem)
		if emoji_size >= rem * 1.9 && emoji_size <= rem * 4.0 {
			best_size = EmojiSizing {
				emojis_per_row,
				emoji_size: Pixels::from(emoji_size as f32),
				list_padding: Pixels::from((emoji_size * list_padding_ratio) as f32),
			};
		} else if emoji_size < rem * 1.9 {
			// Too small, we've gone too far
			break;
		}
	}

	best_size
}

/// Generate the grouped emoji vector that the application is based upon.
pub(crate) fn grouped_emojis() -> Vec<GroupedEmojis> {
	emoji::Group::iter().fold(Vec::from(vec![]), |mut all: Vec<GroupedEmojis>, current_group| {
		let group_emojis: Vec<&'static Emoji> =
			ALL_EMOJI.iter().filter(|e| e.group == current_group).cloned().collect();

		all.push(GroupedEmojis { group: current_group, emojis: group_emojis });

		all
	})
}

/// Searches for emojis based on the provided text query
pub(crate) fn search_emojis(text: &str) -> Vec<&'static Emoji> {
	let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
	matcher.search_best_matching_emojis(text, Some(100)).unwrap()
}
