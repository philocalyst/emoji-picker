use emoji::EmojiEntry;
use gpui::{App, Context, Entity, FocusHandle, Focusable, InteractiveElement, Subscription, Window, prelude::*};
use gpui_component::{IndexPath, gray_800, list::{List, ListEvent, ListState}, v_flex};

use crate::{listgistics::EmojiListDelegate, utilities::calculate_emojis_per_row};

pub(crate) struct Picker {
	/// The current state of focus
	pub(crate) focus_handle: FocusHandle,

	/// The position of the selected emoji, if there is one
	pub(crate) selected_emoji: Option<usize>,

	/// The state of the list
	pub(crate) list_state: Entity<ListState<EmojiListDelegate>>,

	_subscription: Subscription,
}

// Required boilerplate implementation
impl Focusable for Picker {
	fn focus_handle(&self, _: &App) -> FocusHandle { self.focus_handle.clone() }
}

impl Picker {
	pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
		let container_width = window.bounds().size.width.to_f64();

		// Trying to base the emoji size off of the text size for accessiblity reasons,
		// and because emojis are really just... text
		let default_emoji_size = window.rem_size() * 2.0;
		let emojis_per_row = calculate_emojis_per_row(container_width, default_emoji_size);

		// Initialize the list
		let delegate = EmojiListDelegate::new(emojis_per_row, default_emoji_size);
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

		Self { focus_handle: cx.focus_handle(), selected_emoji: None, list_state, _subscription }
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
}

impl Render for Picker {
	fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		v_flex()
			.track_focus(&self.focus_handle(cx))
			.size_full()
			.child(List::new(&self.list_state).bg(gray_800()).p_3().flex_1())
	}
}
