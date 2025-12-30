use std::sync::LazyLock;

use emoji_search;
use gpui::{AnyView, AnyWindowHandle, App, Application, Bounds, Entity, Focusable, KeyBinding, WindowBounds, WindowKind, WindowOptions, actions, prelude::*, px, size};
use gpui_component::{PixelsExt, Root, input::{InputEvent, InputState}, theme::Theme};

use crate::picker::Picker;

mod core_row;
mod grouped_grid;
mod listgistics;
mod picker;
mod utilities;
mod variant_overlay;

struct PickerHandle(Entity<Picker>);
impl gpui::Global for PickerHandle {}

actions!(picker, [JumpToSection]);

actions!(text_input, [Quit,]);

static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
	std::sync::LazyLock::new(|| emoji_search::types::load_emoji_data().unwrap());

static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
	LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));

struct AppState {
	picker: Entity<Picker>,
	window: AnyWindowHandle,
}

impl gpui::Global for AppState {}

fn initialize(cx: &mut App) {
	let rem_size = 16.0;

	let displays = cx.displays();
	let display = displays.first().expect("no display found");
	let display_size = display.bounds().size;

	let initial_width = (display_size.width.as_f32() * 0.25) + (rem_size * 2.0);
	let initial_height = (display_size.height.as_f32() * 0.4) + (rem_size * 4.0);

	let bounds = Bounds::centered(None, size(px(initial_width), px(initial_height)), cx);

	cx.open_window(
		WindowOptions {
			titlebar: None,
			kind: WindowKind::PopUp,
			window_bounds: Some(WindowBounds::Windowed(bounds)),
			..Default::default()
		},
		|window, cx| {
			// Set the theme before creating Root
			cx.set_global(Theme::default());
			gpui_component::init(cx);

			let picker = cx.new(|cx| Picker::new(window, cx));

			window.focus(&picker.read(cx).focus_handle(cx));
			window.activate_window();

			cx.set_global(AppState { picker: picker.clone(), window: window.window_handle() });

			// Wrap InputExample in Root - convert to AnyView
			cx.new(|cx| Root::new(AnyView::from(picker), window, cx))
		},
	)
	.unwrap();
}

fn main() {
	let app = Application::new();

	app.run(|cx: &mut App| {
		initialize(cx);

		cx.on_action(|_: &Quit, cx| {
			inject_text();
			cx.quit();
		});

		cx.on_action(|_: &JumpToSection, cx| {
			// Get the entity from the global store
			let state = cx.global::<AppState>();
			let picker_entity = state.picker.clone();
			let window_handle = state.window;

			// Use the specific window handle instead of searching for the active one
			window_handle
				.update(cx, |_, window, cx| {
					picker_entity.update(cx, |picker, cx| {
						picker.jump_to_section(3, window, cx);
					});
				})
				.unwrap();
		});

		cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
		cx.bind_keys([KeyBinding::new("cmd-l", JumpToSection, None)]);
	});
}

fn inject_text() {
	espanso_inject::get_injector(espanso_inject::InjectorCreationOptions::default())
		.unwrap()
		.send_string("🌞", espanso_inject::InjectionOptions::default())
		.unwrap();
}
