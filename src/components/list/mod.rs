//! Emoji list model, search, and rendering.

use std::sync::LazyLock;

pub(crate) mod types;
pub(crate) mod methods;
pub(crate) mod row;

pub(crate) static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
	LazyLock::new(|| emoji_search::types::load_emoji_data().expect("failed to load emoji data"));

pub(crate) static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
	LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));
