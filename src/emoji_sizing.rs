//! Algorithm for determining emoji row width and count based on container
//! dimensions.

use gpui::Pixels;

pub(crate) struct EmojiSizing {
	pub emojis_per_row: usize,
	pub emoji_size:     Pixels,
	pub list_padding:   Pixels,
}

pub(crate) fn calculate_emoji_sizing(container_width: f64, rem_size: Pixels) -> EmojiSizing {
	let rem = rem_size.to_f64();

	const MIN_EMOJIS_PER_ROW: usize = 8;
	const MAX_EMOJIS_PER_ROW: usize = 20;

	let list_padding_ratio = 0.25;

	// Each emoji has p_2() padding which is 0.25rem on each side
	let emoji_padding_per_side = rem * 0.25;
	let total_emoji_padding = emoji_padding_per_side * 2.0;

	let mut best_size = EmojiSizing {
		emojis_per_row: MIN_EMOJIS_PER_ROW,
		emoji_size:     Pixels::from(0.0),
		list_padding:   Pixels::from(0.0),
	};

	for emojis_per_row in MIN_EMOJIS_PER_ROW..=MAX_EMOJIS_PER_ROW {
		let denominator = (2.0 * list_padding_ratio) + emojis_per_row as f64;
		let numerator = container_width - (emojis_per_row as f64 * total_emoji_padding);
		let emoji_size = numerator / denominator;

		if emoji_size >= rem * 2.2 && emoji_size <= rem * 4.0 {
			best_size = EmojiSizing {
				emojis_per_row,
				emoji_size: Pixels::from((emoji_size * 0.90) as f32),
				list_padding: Pixels::from((emoji_size * list_padding_ratio) as f32),
			};
		} else if emoji_size < rem * 1.9 {
			break;
		}
	}

	best_size
}
