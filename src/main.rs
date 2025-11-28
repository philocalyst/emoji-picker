use std::sync::LazyLock;

use emoji_search;
use gpui::{App, Application, Bounds, Entity, Focusable, KeyBinding, WindowBounds, WindowOptions, actions, prelude::*, px, size};
use gpui_component::{Root, VirtualListScrollHandle, input::{InputEvent, InputState}, scroll::ScrollbarState, theme::Theme};

use crate::picker::Picker;

mod emojir;
mod input;
mod picker;
mod utils;
mod variant_overlay;

actions!(text_input, [Quit,]);

static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
	std::sync::LazyLock::new(|| emoji_search::types::load_emoji_data().unwrap());

static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
	LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));

fn main() {
	let app = Application::new();

	app.run(|cx: &mut App| {
		let bounds = Bounds::centered(None, size(px(450.0), px(450.0)), cx);

		cx.open_window(
			WindowOptions {
				titlebar: None,
				window_bounds: Some(WindowBounds::Windowed(bounds)),
				..Default::default()
			},
			|window, cx| {
				// Set the theme before creating Root
				cx.set_global(Theme::default());
				gpui_component::init(cx);

				let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Type here..."));

				window.focus(&input_state.read(cx).focus_handle(cx));

				// Subscribe with correct closure signature
				cx.subscribe(
					&input_state,
					|_subscriber: Entity<InputState>, _event: &InputEvent, cx: &mut App| {
						let text = _subscriber.read(cx);
						eprintln!("Input event: {:?}", text.text());
					},
				)
				.detach();

				let input_example = cx.new(|cx| Picker {
					emojis:            vec![],
					scroll_handle:     VirtualListScrollHandle::new(),
					scroll_state:      ScrollbarState::default(),
					input_state:       input_state.clone(),
					recent_keystrokes: vec![],
					focus_handle:      cx.focus_handle(),
					selected_emoji:    None,
				});

				// Wrap InputExample in Root - convert to AnyView
				cx.new(|cx| Root::new(input_example.into(), window, cx))
			},
		)
		.unwrap();

		cx.on_action(|_: &Quit, cx| cx.quit());
		cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
	});
}
