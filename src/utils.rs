use std::{path::PathBuf, rc::Rc};

use emoji::Emoji;
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

/// Generates skin tone variant strings for a given emoji
pub(crate) fn generate_skin_tone_variants(emoji_glyph: &str) -> Option<Vec<String>> {
	let supports_skin_tone = emoji_glyph.chars().any(|c| {
		let code = c as u32;
		// People ranges
		(0x1F385..=0x1F93E).contains(&code) || // Various people
        (0x1F442..=0x1F4AA).contains(&code) || // Body parts
        (0x1F574..=0x1F57A).contains(&code) || // More people
        (0x1F590..=0x1F595).contains(&code) || // Hand gestures
        (0x1F645..=0x1F64F).contains(&code) || // Person gestures
        (0x1F6A3..=0x1F6CC).contains(&code) || // Activities
        (0x1F90C..=0x1F93E).contains(&code) || // Supplemental
        (0x1F9B5..=0x1F9BB).contains(&code) || // Body parts extended
        (0x1FAC3..=0x1FAF8).contains(&code) // Extended pictographs
	});

	if !supports_skin_tone {
		return None;
	}

	let skin_tone_modifiers = ["\u{1F3FB}", "\u{1F3FC}", "\u{1F3FD}", "\u{1F3FE}", "\u{1F3FF}"];

	Some(skin_tone_modifiers.iter().map(|modifier| format!("{}{}", emoji_glyph, modifier)).collect())
}

/// Searches for emojis based on the provided text query
pub(crate) fn search_emojis(text: &str) -> Vec<&'static Emoji> {
	match text {
		"" => emoji::lookup_by_glyph::iter_emoji().filter(|emoji| !emoji.name.contains(":")).collect(),
		_ => {
			let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
			matcher.search_best_matching_emojis(text, Some(1000)).unwrap()
		}
	}
}
