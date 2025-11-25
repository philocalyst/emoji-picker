use std::rc::Rc;

use emoji::Emoji;
use gpui::{Pixels, Size, size};

use crate::SEARCHER;

/// Calculates the number of emojis that fit per row based on container width
pub(crate) fn calculate_emojis_per_row(container_width: f64, emoji_size: Pixels) -> usize {
    let emojis_per_row = (container_width / emoji_size.to_f64()).floor() as usize;
    emojis_per_row.max(1)
}

/// Generates row sizes for the virtual list based on emoji count and layout
pub(crate) fn generate_row_sizes(
    emoji_count: usize,
    emojis_per_row: usize,
    container_width: f64,
    emoji_size: Pixels,
) -> Rc<Vec<Size<Pixels>>> {
    let row_count = (emoji_count + emojis_per_row - 1) / emojis_per_row;
    Rc::new(
        (0..row_count)
            .map(|_| size(container_width.into(), emoji_size))
            .collect(),
    )
}

/// Generates skin tone variant strings for a given emoji
pub(crate) fn generate_skin_tone_variants(emoji_glyph: &str) -> Vec<String> {
    let skin_tone_modifiers = [
        "\u{1F3FB}", // Light Skin Tone
        "\u{1F3FC}", // Medium-Light Skin Tone
        "\u{1F3FD}", // Medium Skin Tone
        "\u{1F3FE}", // Medium-Dark Skin Tone
        "\u{1F3FF}", // Dark Skin Tone
    ];

    skin_tone_modifiers
        .iter()
        .map(|modifier| {
            let mut variant_glyph = String::with_capacity(emoji_glyph.len() + modifier.len());
            variant_glyph.push_str(emoji_glyph);
            variant_glyph.push_str(modifier);
            variant_glyph
        })
        .collect()
}

/// Searches for emojis based on the provided text query
pub(crate) fn search_emojis(text: &str) -> Vec<&'static Emoji> {
    match text {
        "" => emoji::lookup_by_glyph::iter_emoji()
            .filter(|emoji| !emoji.name.contains(":"))
            .collect(),
        _ => {
            let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
            matcher
                .search_best_matching_emojis(text, Some(1000))
                .unwrap()
        }
    }
}
