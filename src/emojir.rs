use std::rc::Rc;

use emoji::Emoji;
use gpui::{Div, Entity, InteractiveElement, IntoElement, ParentElement, Pixels, Size, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, rems};
use gpui_component::{VirtualListScrollHandle, v_virtual_list};

use crate::{picker::Picker, variant_overlay};

impl Picker {
	/// Renders a single emoji button
	pub(crate) fn render_button(
		emoji_idx: usize,
		emoji: &Emoji,
		selected_emoji: Option<usize>,
		entity: Entity<Picker>,
	) -> impl IntoElement {
		div()
			.id(emoji_idx)
			.relative()
			.child(emoji.glyph)
			.when(selected_emoji == Some(emoji_idx), |parent| {
				parent.child(div().absolute().top_0().left_0().child(variant_overlay::render(emoji)))
			})
			.cursor_pointer()
			.on_click({
				let emoji = emoji.to_owned();
				let entity = entity.clone(); // Clone for the closure
				move |_event, _, ctx| {
					// Update using the correct GPUI API
					entity.update(ctx, |picker, cx| {
						picker.selected_emoji = Some(emoji_idx);
						cx.notify();
					});
					println!("{emoji:?}");
				}
			})
	}

	/// Renders a row of emojis for the virtual list
	pub(crate) fn render_row(
		start_idx: usize,
		end_idx: usize,
		emojis: &[&Emoji],
		selected_emoji: Option<usize>,
		entity: Entity<Picker>,
	) -> Div {
		div().flex().flex_row().children((start_idx..end_idx).map(|emoji_idx| {
			let emoji = emojis[emoji_idx];
			Self::render_button(emoji_idx, emoji, selected_emoji, entity.clone())
		}))
	}

	/// Renders the emoji grid with virtual scrolling
	pub(crate) fn render_grid(
		entity: Entity<Picker>,
		emojis: Vec<&'static Emoji>,
		emojis_per_row: usize,
		selected_emoji: Option<usize>,
		row_sizes: Rc<Vec<Size<Pixels>>>,
		emoji_text_size: f32,
		scroll_handle: &VirtualListScrollHandle,
	) -> impl IntoElement {
		let entity_clone = entity.clone();

		v_virtual_list(
			entity,
			"emojis",
			row_sizes,
			move |_container: &mut Picker, range: std::ops::Range<usize>, _window, _cx| {
				range
					.map(|row_idx| {
						let start_idx = row_idx * emojis_per_row;
						let end_idx = (start_idx + emojis_per_row).min(emojis.len());
						Self::render_row(start_idx, end_idx, &emojis, selected_emoji, entity_clone.clone())
					})
					.collect()
			},
		)
		.text_size(rems(emoji_text_size))
		.track_scroll(scroll_handle)
		.h_full()
	}
}
