use emoji::{Emoji, EmojiEntry};
use gpui::{App, Context, Entity, FocusHandle, Focusable, InteractiveElement, Subscription, Window, prelude::*};
use gpui_component::{IndexPath, input::InputState, list::{List, ListEvent, ListState}, v_flex};

use crate::{input, scrollable_groupings::EmojiListDelegate, utilities::calculate_emojis_per_row};

pub(crate) struct Picker {
	pub(crate) input_state:  Entity<InputState>,
	pub(crate) focus_handle: FocusHandle,

	/// The position of the selected emoji, if there is one
	pub(crate) selected_emoji: Option<usize>,
	pub(crate) list_state:     Entity<ListState<EmojiListDelegate>>,
	_subscription:             Subscription,
}

impl Focusable for Picker {
	fn focus_handle(&self, _: &App) -> FocusHandle { self.focus_handle.clone() }
}

impl Picker {
	pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
		let container_width = window.bounds().size.width.to_f64();
		let default_emoji_size = window.rem_size() * 2.0;
		let emojis_per_row = calculate_emojis_per_row(container_width, default_emoji_size);

		let delegate = EmojiListDelegate::new(emojis_per_row);
		let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

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
							// TODO: Actually insert/use the emoji
						}
					}
				}
				ListEvent::Cancel => {
					println!("Cancelled emoji selection");
				}
			}
		});

		Self {
			input_state: cx.new(|cx| InputState::new(window, cx)),
			focus_handle: cx.focus_handle(),
			selected_emoji: None,
			list_state,
			_subscription,
		}
	}

	fn index_path_to_emoji_index(&self, ix: IndexPath, cx: &App) -> Option<usize> {
		dbg!(&ix);
		let delegate = self.list_state.read(cx).delegate();

		// Calculate global emoji index from IndexPath
		let mut global_idx = 0;

		// Add all emojis from previous sections
		for section in 0..ix.section {
			if section >= delegate.grouped_emojis.len() {
				return None;
			}
			global_idx += delegate.grouped_emojis[section].emojis.len();
		}

		// Add emojis from rows within the current section
		if ix.section >= delegate.grouped_emojis.len() {
			return None;
		}
		let emojis_per_row = delegate.emojis_per_row;
		let starting_row = ix.row * emojis_per_row;

		global_idx += starting_row;

		// Add the already acknowledged columns
		global_idx += ix.column;

		Some(global_idx)
	}

	fn get_emoji_at_index(&self, idx: usize, cx: &App) -> Option<&'static EmojiEntry> {
		let delegate = self.list_state.read(cx).delegate();

		let mut current_idx = 0;
		for group in &delegate.grouped_emojis {
			let group_len = group.emojis.len();
			if idx < current_idx + group_len {
				return Some(group.emojis[idx - current_idx]);
			}
			current_idx += group_len;
		}

		None
	}
}

impl Render for Picker {
	fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		v_flex()
			.track_focus(&self.focus_handle(cx))
			.size_full()
			.child(input::render(&self.input_state))
			.child(List::new(&self.list_state).flex_1().w_full())
	}
}
