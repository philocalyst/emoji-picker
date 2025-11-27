use std::{path::PathBuf, rc::Rc};

use emoji::{Emoji, EmojiEntry};
use gpui::{Pixels, Size, size};
use unicode_segmentation::UnicodeSegmentation;

use crate::SEARCHER;

/// Calculates the number of emojis that fit per row based on container width
pub(crate) fn calculate_emojis_per_row(container_width: f64, emoji_size: Pixels) -> usize {
	let emojis_per_row = (container_width / emoji_size.to_f64()).floor() as usize;
	emojis_per_row.max(1)
}

/// Returns an array of paths pointing to SVG's representing the various emoji
/// categories
pub(crate) fn get_bar_icons() -> Vec<PathBuf> {
	// Rightmost words from your list
	const DESIRED_CATAGORIES: [&str; 10] = [
		"activity",         // activities
		"rat",              // animals_and_nature
		"component",        // component
		"flag",             // flags
		"utensils-crossed", // food_and_drink
		"target",           // objects
		"person-standing",  // people_and_body
		"smile-plus",       // smileys_and_emotion
		"shell",            // symbols
		"ship",             // travel_and_places
	];

	let base = PathBuf::from("./lucide/icons");

	DESIRED_CATAGORIES.iter().map(|name| base.join(format!("{name}.svg"))).collect()
}

/// Generates row sizes for the virtual list based on emoji count and layout
pub(crate) fn generate_row_sizes(
	emoji_count: usize,
	emojis_per_row: usize,
	container_width: f64,
	emoji_size: Pixels,
) -> Rc<Vec<Size<Pixels>>> {
	let row_count = (emoji_count + emojis_per_row - 1) / emojis_per_row;
	Rc::new((0..row_count).map(|_| size(container_width.into(), emoji_size)).collect())
}

/// Searches for emojis based on the provided text query
pub(crate) fn search_emojis(text: &str) -> Vec<&'static EmojiEntry> {
	match text {
		"" => emoji::lookup_by_glyph::iter_emoji().filter(|emoji| emoji.tones().is_some()).collect(),
		_ => {
			let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
			matcher.search_best_matching_emojis(text, Some(1000)).unwrap()
		}
	}
}
