use std::{process::Command, sync::LazyLock, thread::sleep, time::Duration};

use emoji_search;
use gpui::{AnyView, App, Application, Bounds, Entity, Focusable, KeyBinding, WindowBounds, WindowOptions, actions, prelude::*, px, size};
use gpui_component::{Root, input::{InputEvent, InputState}, theme::Theme};

use crate::picker::Picker;

mod core_row;
mod grouped_grid;
mod picker;
mod utilities;
mod variant_overlay;

actions!(text_input, [Quit,]);

static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
	std::sync::LazyLock::new(|| emoji_search::types::load_emoji_data().unwrap());

static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
	LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));

fn main() {
	let app = Application::new();

	app.run(|cx: &mut App| {
		let bounds = Bounds::centered(None, size(px(550.0), px(550.0)), cx);

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

				window.activate_window();
				// Subscribe with correct closure signature
				cx.subscribe(
					&input_state,
					|_subscriber: Entity<InputState>, _event: &InputEvent, cx: &mut App| {
						let text = _subscriber.read(cx);
						eprintln!("Input event: {:?}", text.text());
					},
				)
				.detach();

				let input_example = cx.new(|cx| Picker::new(window, cx));

				// Wrap InputExample in Root - convert to AnyView
				cx.new(|cx| Root::new(AnyView::from(input_example), window, cx))
			},
		)
		.unwrap();

		cx.on_action(|_: &Quit, cx| {
			inject_text();
			cx.quit();
		});
		cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
	});
}

fn inject_text() {
	let output = Command::new("osascript")
		.args(&["-e", &format!("tell application \"{}\" to activate", "ghostty")])
		.output()
		.unwrap();

	sleep(Duration::from_secs(1));

	espanso_inject::get_injector(espanso_inject::InjectorCreationOptions::default())
		.unwrap()
		.send_string("🌞", espanso_inject::InjectionOptions::default())
		.unwrap();
}
