use emoji::Emoji;
use gpui::{App, Context, Edges, Entity, FocusHandle, Focusable, InteractiveElement, Pixels, Subscription, Window, actions, prelude::*, px};
use gpui_component::{IndexPath, StyledExt, gray_800, list::{List, ListEvent, ListState}, purple_400, v_flex};
use nonempty::NonEmpty;

use crate::{Cancel, JumpToSection, PopoverState, Quit, RotateTones, SelectedEmoji, ToneIndex, insert_emoji, listgistics::EmojiListDelegate, utilities::calculate_emoji_sizing};

actions!(picker, [
	MoveUp,
	MoveDown,
	MoveLeft,
	MoveRight,
	SelectCurrent,
	OpenSecondary,
	FocusSearch
]);

pub(crate) struct Picker {
	/// The current state of focus
	pub(crate) focus_handle: FocusHandle,

	/// Body focus handle
	pub(crate) body_focus_handle: FocusHandle,

	/// The position of the selected emoji, if there is one
	pub(crate) selected_emoji: Option<&'static Emoji>,

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

		let _last_selected = cx.default_global::<SelectedEmoji>().0.clone();

		let body_focus_handle = cx.focus_handle();

		// Initialize the list
		let delegate =
			EmojiListDelegate::new(sizing.emojis_per_row, sizing.emoji_size, body_focus_handle.clone());
		let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

		// Handle the events on the list
		let _subscription = cx.subscribe(&list_state, |picker, _, ev: &ListEvent, cx| {
			match ev {
				ListEvent::Select(ix) => {
					// Convert IndexPath to global emoji index
					if let Some(emoji) = picker.get_emoji_at_path(*ix, cx) {
						picker.selected_emoji = Some(emoji);
						cx.set_global(SelectedEmoji(Some(NonEmpty::new(emoji.clone()))));
					}
				}
				ListEvent::Confirm(ix) => {
					if let Some(emoji) = picker.get_emoji_at_path(*ix, cx) {
						picker.selected_emoji = Some(emoji);
						// Get the actual emoji and do something with it
					}
				}
				ListEvent::Cancel => {
					println!("Cancelled emoji selection");
				}
			}
		});

		Self {
			focus_handle: cx.focus_handle(),
			body_focus_handle,
			selected_emoji: None,
			list_state,
			padding: sizing.list_padding,
			_subscription,
		}
	}

	fn get_emoji_at_path(&self, ix: IndexPath, cx: &App) -> Option<&'static Emoji> {
		let delegate = self.list_state.read(cx).delegate();
		delegate
			.emoji_legions
			.get(ix.section)?
			.emojis
			.get(ix.row * delegate.emojis_per_row + ix.column)
			.map(|e| *e)
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

	fn update_selection<F>(&self, window: &mut Window, cx: &mut App, f: F)
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

	fn select_current(&self, _window: &mut Window, cx: &mut App) {
		let selected_emoji = self
			.list_state
			.read(cx)
			.delegate()
			.selected_index
			.and_then(|ix| self.get_emoji_at_path(ix, cx));

		if let Some(emoji) = selected_emoji {
			insert_emoji(emoji.glyph);
			cx.shutdown();
		}
	}

	fn open_secondary(&self, _window: &mut Window, cx: &mut App) {
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

	fn focus_search(&self, window: &mut Window, cx: &mut App) {
		self.list_state.update(cx, |input, cx| {
			input.focus(window, cx);
		});
	}

	fn cancel(&self, window: &mut Window, cx: &mut App) {
		let popover_state = cx.global::<PopoverState>();
		if popover_state.open_emoji.is_some() {
			cx.update_global::<PopoverState, _>(|state, _| {
				state.open_emoji = None;
			});
			return;
		}

		if self.list_state.read(cx).focus_handle(cx).is_focused(window) {
			// Focus list after blurring search
			// The Picker component tracks focus.
			self.body_focus_handle.focus(window);
			return;
		}

		cx.dispatch_action(&Quit);
	}
}

impl Render for Picker {
	fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let mut emoji_edges = Edges::all(px(8.));
		emoji_edges.right = px(-4.); // Some weird leftover padding on the right

		v_flex()
			.bg(gray_800())
			.text_color(purple_400())
			.on_action(cx.listener(move |_, directive: &RotateTones, _, cx| {
				let current_index = cx.default_global::<ToneIndex>();

				current_index.rotate(directive.direction.clone());

				// redraw
				cx.notify();
			}))
			.on_action(cx.listener(|this, section: &JumpToSection, window, cx| {
				this.jump_to_section(section.number, window, cx);
			}))
			.on_action(cx.listener(|this, _: &MoveUp, window, cx| {
				this.update_selection(window, cx, |d| d.move_up());
			}))
			.on_action(cx.listener(|this, _: &MoveDown, window, cx| {
				this.update_selection(window, cx, |d| d.move_down());
			}))
			.on_action(cx.listener(|this, _: &MoveLeft, window, cx| {
				this.update_selection(window, cx, |d| d.move_left());
			}))
			.on_action(cx.listener(|this, _: &MoveRight, window, cx| {
				this.update_selection(window, cx, |d| d.move_right());
			}))
			.on_action(cx.listener(|this, _: &SelectCurrent, window, cx| {
				this.select_current(window, cx);
			}))
			.on_action(cx.listener(|this, _: &OpenSecondary, window, cx| {
				this.open_secondary(window, cx);
			}))
			.on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
				this.focus_search(window, cx);
			}))
			.on_action(cx.listener(|this, _: &Cancel, window, cx| {
				this.cancel(window, cx);
			}))
			.track_focus(&self.focus_handle(cx))
			.key_context("Picker")
			.size_full()
			.child(List::new(&self.list_state).scrollbar_visible(false).paddings(emoji_edges))
	}
}
