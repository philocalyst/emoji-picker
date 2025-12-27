use emoji::EmojiEntry;
use gpui::Pixels;

use crate::SEARCHER;

/// Calculates the number of emojis that fit per row based on container width
pub(crate) fn calculate_emojis_per_row(container_width: f64, emoji_size: Pixels) -> usize {
	// We're calculating this as based upon the container width as emojis are
	// largely static, and we're handling layout by rows, not relying on native
	// wrapping capabilities.
	let emojis_per_row = (container_width / emoji_size.to_f64()).floor() as usize;

	// There needs to be at least one emoji in a row, regardless of size.
	emojis_per_row.max(1)
}

/// Searches for emojis based on the provided text query
pub(crate) fn search_emojis(text: &str) -> Vec<&'static EmojiEntry> {
	match text {
		"" => emoji::lookup_by_glyph::iter_emoji().collect(),
		_ => {
			let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
			matcher.search_best_matching_emojis(text, Some(1000)).unwrap()
		}
	}
}
