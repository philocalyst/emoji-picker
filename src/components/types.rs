//! Core types and global state for the picker UI.

use emoji::Emoji;
use gpui::{Entity, FocusHandle, Pixels};
use nonempty::NonEmpty;

use crate::components::list::types::EmojiListDelegate;

use gpui_component::list::ListState;

pub(crate) struct Picker {
	pub(crate) focus_handle: FocusHandle,
	pub(crate) body_focus_handle: FocusHandle,
	pub(crate) selected_emoji: Option<&'static Emoji>,
	pub(crate) list_state: Entity<ListState<EmojiListDelegate>>,
	pub(crate) _padding: Pixels,
	pub(crate) _subscription: gpui::Subscription,
}

impl gpui::Focusable for Picker {
	fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
		self.focus_handle.clone()
	}
}

pub(crate) struct SelectedEmoji(pub Option<NonEmpty<Emoji>>);
impl gpui::Global for SelectedEmoji {}

impl Default for SelectedEmoji {
	fn default() -> Self {
		Self(None)
	}
}

pub(crate) struct ToneIndex(pub u8);
impl gpui::Global for ToneIndex {}

impl ToneIndex {
	pub const MAX: u8 = 6;

	pub fn rotate(&mut self, direction: crate::keys::Direction) {
		use crate::keys::Direction;
		let place = &mut self.0;
		*place = match direction {
			Direction::Forward => (*place + 1) % Self::MAX,
			Direction::Backward => (*place + Self::MAX - 1) % Self::MAX,
		};
	}
}

impl Default for ToneIndex {
	fn default() -> Self {
		Self(0)
	}
}

#[derive(Clone, Copy)]
pub(crate) struct PopoverState {
	pub open_emoji: Option<&'static Emoji>,
}

impl Default for PopoverState {
	fn default() -> Self {
		Self { open_emoji: None }
	}
}

impl gpui::Global for PopoverState {}
