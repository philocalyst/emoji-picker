use std::{ops::Range, rc::Rc, sync::Mutex};

use emoji::{Emoji, EmojiEntry, Group};
use gpui::{Div, Entity, InteractiveElement, IntoElement, ParentElement, Pixels, Size, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, rems};
use gpui_component::{VirtualListScrollHandle, v_virtual_list};

use crate::{picker::Picker, variant_overlay};

impl Picker {
	/// Renders a single emoji button
	pub(crate) fn render_button(
		emoji: &EmojiEntry,
		selected_emoji: Option<EmojiEntry>,
		entity: Entity<Picker>,
	) -> impl IntoElement {
		let emoji_name = emoji.emoji().name;
		div()
			.id(emoji_name)
			.relative()
			.child(emoji.emoji().glyph)
			.when(selected_emoji == Some(*emoji), |parent| {
				parent.child(div().absolute().top_0().left_0().child(variant_overlay::render(emoji)))
			})
			.cursor_pointer()
			.on_click({
				let emoji = emoji.to_owned();
				let entity = entity.clone(); // Clone for the closure
				move |_event, _, ctx| {
					// Update using the correct GPUI API
					entity.update(ctx, |picker, cx| {
						picker.selected_emoji = Some(emoji);
						cx.notify();
					});
					println!("{emoji:?}");
				}
			})
	}

	/// Renders a section of emojis
	pub(crate) fn render_section(
		emojis_in_use: &[&EmojiEntry],
		selected_emoji: Option<EmojiEntry>,
		entity: Entity<Picker>,
	) -> Div {
		div().flex().flex_row().children(
			emojis_in_use.iter().map(|emoji| Self::render_button(emoji, selected_emoji, entity.clone())),
		)
	}

	/// Renders the emoji grid with virtual scrolling
	pub(crate) fn render_grid(
		picker: Entity<Picker>,
		emojis: Vec<&'static EmojiEntry>,
		emojis_per_row: usize,
		selected_emoji: Option<EmojiEntry>,
		row_sizes: Rc<Vec<Size<Pixels>>>,
		emoji_text_size: f32,
		scroll_handle: &VirtualListScrollHandle,
	) -> impl IntoElement {
		// This list is not a long list of emojis, it's a long list of rows of emojis
		v_virtual_list(
			picker.clone(),
			"emojis", // Accessible name
			row_sizes,
			move |_picker, active_sections: Range<usize>, _window, _ctx| {
				active_sections
					.map(|section_index| {
						// Get the active group
						let current_group: Group = Group::ALL[section_index];

						// Only get those that we need
						let relevant_emojis: Vec<&EmojiEntry> =
							emojis.iter().filter(|emoji| emoji.emoji().group == current_group).cloned().collect();

						Self::render_section(&relevant_emojis, selected_emoji, picker.clone())
					})
					.collect()
			},
		)
		.text_size(rems(emoji_text_size))
		.track_scroll(scroll_handle)
		.h_full()
	}
}
