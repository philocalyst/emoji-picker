//! Picker behavior: construction, navigation, selection, and search focus.

use std::{thread, time::Duration};

use emoji::Emoji;
use gpui::{App, AppContext, BorrowAppContext, Context, Focusable, Window};
use gpui_component::{
	IndexPath,
	list::{ListEvent, ListState},
};
use nonempty::NonEmpty;
use tracing::debug;

use crate::{
	components::{
		list::types::EmojiListDelegate,
		types::{Picker, PopoverState, SelectedEmoji},
	},
	emoji_sizing::calculate_emoji_sizing,
	insert::insert_emoji,
	keys::Quit,
};

impl Picker {
	pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
		let container_width = window.bounds().size.width.to_f64();
		let rem_size = window.rem_size();

		let sizing = calculate_emoji_sizing(container_width, rem_size);

		let _last_selected = cx.default_global::<SelectedEmoji>().0.clone();

		let body_focus_handle = cx.focus_handle();

		let delegate =
			EmojiListDelegate::new(sizing.emojis_per_row, sizing.emoji_size, body_focus_handle.clone());
		let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

		let _subscription = cx.subscribe(&list_state, |picker, _, ev: &ListEvent, cx| match ev {
			ListEvent::Select(ix) => {
				if let Some(emoji) = picker.get_emoji_at_path(*ix, cx) {
					debug!(emoji = emoji.name, "emoji selected");
					picker.selected_emoji = Some(emoji);
					cx.set_global(SelectedEmoji(Some(NonEmpty::new(emoji.clone()))));
				}
			}
			ListEvent::Confirm(ix) => {
				if let Some(emoji) = picker.get_emoji_at_path(*ix, cx) {
					picker.selected_emoji = Some(emoji);
				}
			}
			ListEvent::Cancel => {
				debug!("emoji selection cancelled");
			}
		});

		Self {
			focus_handle: cx.focus_handle(),
			body_focus_handle,
			selected_emoji: None,
			list_state,
			_padding: sizing.list_padding,
			_subscription,
		}
	}

	pub(crate) fn get_emoji_at_path(&self, ix: IndexPath, cx: &App) -> Option<&'static Emoji> {
		let delegate = self.list_state.read(cx).delegate();
		delegate
			.emoji_legions
			.get(ix.section)?
			.emojis
			.get(ix.row * delegate.emojis_per_row + ix.column)
			.map(|e| *e)
	}

	pub(crate) fn jump_to_section(&self, section: usize, window: &mut Window, cx: &mut App) {
		cx.update_entity(&self.list_state, |list, cx| {
			list.scroll_to_item(
				IndexPath { section, row: 0, column: 0 },
				gpui::ScrollStrategy::Center,
				window,
				cx,
			);
		});
	}

	pub(crate) fn update_selection<F>(&self, window: &mut Window, cx: &mut App, f: F)
	where
		F: FnOnce(&mut EmojiListDelegate),
	{
		self.list_state.update(cx, |list, cx| {
			f(list.delegate_mut());
			if let Some(ix) = list.delegate().selected_index {
				list.scroll_to_item(ix, gpui::ScrollStrategy::Center, window, cx);
			}
			cx.notify();
		});
	}

	pub(crate) fn select_current(&self, _window: &mut Window, cx: &mut App) {
		let selected_emoji = self
			.list_state
			.read(cx)
			.delegate()
			.selected_index
			.and_then(|ix| self.get_emoji_at_path(ix, cx));

		if let Some(emoji) = selected_emoji {
			insert_emoji(emoji.glyph, cx);
			thread::sleep(Duration::from_millis(100));
			cx.quit();
		}
	}

	pub(crate) fn open_secondary(&self, _window: &mut Window, cx: &mut App) {
		let selected_emoji = self
			.list_state
			.read(cx)
			.delegate()
			.selected_index
			.and_then(|ix| self.get_emoji_at_path(ix, cx));

		if let Some(emoji) = selected_emoji {
			if emoji.skin_tones.is_some() {
				cx.update_global::<PopoverState, _>(|state, _cx| {
					state.open_emoji = Some(emoji);
				});
			}
		}
	}

	pub(crate) fn focus_search(&self, window: &mut Window, cx: &mut App) {
		self.list_state.update(cx, |input, cx| {
			input.focus(window, cx);
		});
	}

	pub(crate) fn cancel(&self, window: &mut Window, cx: &mut App) {
		let popover_state = cx.global::<PopoverState>();
		if popover_state.open_emoji.is_some() {
			cx.update_global::<PopoverState, _>(|state, _| {
				state.open_emoji = None;
			});
			return;
		}

		if self.list_state.read(cx).focus_handle(cx).is_focused(window) {
			self.body_focus_handle.focus(window);
			return;
		}

		cx.dispatch_action(&Quit);
	}
}
