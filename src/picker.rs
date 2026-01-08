use std::fs::write;

use emoji::EmojiEntry;
use gpui::{App, Context, Entity, FocusHandle, Focusable, InteractiveElement, Pixels, Subscription, Window, prelude::*, transparent_black};
use gpui_component::{ActiveTheme, IndexPath, list::{List, ListEvent, ListState}, v_flex};

use crate::{JumpToSection, RotateTonesBackward, RotateTonesForward, ToneIndex, listgistics::EmojiListDelegate, utilities::calculate_emoji_sizing};

pub(crate) struct Picker {
	/// The current state of focus
	pub(crate) focus_handle: FocusHandle,

	/// The position of the selected emoji, if there is one
	pub(crate) selected_emoji: Option<usize>,

	/// The state of the list
	pub(crate) list_state: Entity<ListState<EmojiListDelegate>>,

	padding: Pixels,

	_subscription: Subscription,
}

// Required boilerplate implementation
impl Focusable for Picker {
	fn focus_handle(&self, _: &App) -> FocusHandle { self.focus_handle.clone() }
}

impl Picker {
	pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
		let container_width = window.bounds().size.width.to_f64();
		let rem_size = window.rem_size();

		// The number of emojis per row is responsive to the container width,
		// but clamped to a reasonable range.
		let sizing = calculate_emoji_sizing(container_width, rem_size);

		// Initialize the list
		let delegate = EmojiListDelegate::new(sizing.emojis_per_row, sizing.emoji_size);
		let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

		// Handle the events on the list
		let _subscription = cx.subscribe(&list_state, |picker, _, ev: &ListEvent, cx| {
			match ev {
				ListEvent::Select(ix) => {
					// Convert IndexPath to global emoji index
					if let Some(global_idx) = picker.index_path_to_emoji_index(*ix, cx) {
						picker.selected_emoji = Some(global_idx);
						println!("Selected emoji index: {}", global_idx);
					}
				}
				ListEvent::Confirm(ix) => {
					if let Some(global_idx) = picker.index_path_to_emoji_index(*ix, cx) {
						picker.selected_emoji = Some(global_idx);
						// Get the actual emoji and do something with it
						if let Some(emoji) = picker.get_emoji_at_index(global_idx, cx) {
							println!("Confirmed emoji: {:?}", emoji);
							// TODO: Actually insert/use the emoji, and quit the application
						}
					}
				}
				ListEvent::Cancel => {
					println!("Cancelled emoji selection");
				}
			}
		});

		Self {
			focus_handle: cx.focus_handle(),
			selected_emoji: None,
			list_state,
			padding: sizing.list_padding,
			_subscription,
		}
	}

	fn index_path_to_emoji_index(&self, ix: IndexPath, cx: &App) -> Option<usize> {
		// Get our representative
		let list = self.list_state.read(cx).delegate();

		// Calculate global emoji index from IndexPath using an updating standin
		let mut global_idx = 0;

		// Add all emojis from previous sections
		global_idx +=
			list.emoji_legions.iter().take(ix.section).map(|legion| legion.emojis.len()).sum::<usize>();

		// Total, respcting the existing row progress
		let starting_row = ix.row * list.emojis_per_row;
		global_idx += starting_row;

		// Add the columns up to this point
		global_idx += ix.column;

		Some(global_idx)
	}

	// Get an emoji at an absolute, global index (no index path)
	fn get_emoji_at_index(&self, idx: usize, cx: &App) -> Option<&'static EmojiEntry> {
		let delegate = self.list_state.read(cx).delegate();

		delegate.emoji_legions.iter().flat_map(|group| &group.emojis).nth(idx).copied()
	}

	pub(crate) fn jump_to_section(&self, section: usize, window: &mut gpui::Window, cx: &mut App) {
		cx.update_entity(&self.list_state, |list, cx| {
			list.scroll_to_item(
				IndexPath { section, row: 0, column: 0 },
				gpui::ScrollStrategy::Center,
				window,
				cx,
			);
		});
	}
}

fn rotate_tones(current_index: &mut ToneIndex, up: bool) {
	const MAX: u8 = 6;

	if up {
		current_index.0 = (current_index.0 + 1) % MAX;
	} else {
		current_index.0 = (current_index.0 + MAX - 1) % MAX;
	}
}

impl Render for Picker {
	fn render(&mut self, win: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		v_flex()
			.bg(cx.theme().colors.background)
			.text_color(cx.theme().colors.foreground)
			.p_1()
			.on_action(cx.listener(|_, _: &RotateTonesForward, _, cx| {
				let current_index = cx.default_global::<ToneIndex>();

				rotate_tones(current_index, true);

				// redraw
				cx.notify();
			}))
			.on_action(cx.listener(|_, _: &RotateTonesBackward, _, cx| {
				let current_index = cx.default_global::<ToneIndex>();

				rotate_tones(current_index, false);

				// redraw
				cx.notify();
			}))
			.on_action(cx.listener(|this, section: &JumpToSection, window, cx| {
				this.jump_to_section(section.number, window, cx);
			}))
			.track_focus(&self.focus_handle(cx))
			.size_full()
			.child(
				List::new(&self.list_state).scrollbar_visible(false).text_xl().p(self.padding).flex_1(),
			)
	}
}
