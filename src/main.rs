use std::{sync::LazyLock, time::Duration};

use emoji_search;
#[cfg(target_os = "macos")]
use global_hotkey::hotkey::Modifiers;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, HotKey}};
use gpui::{AnyView, AnyWindowHandle, App, Application, Bounds, Entity, Focusable, KeyBinding, WindowBounds, WindowKind, WindowOptions, actions, prelude::*, px, size};
use gpui_component::{ActiveTheme, PixelsExt, Root, ThemeRegistry, theme::{self, Theme, ThemeMode}};

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
actions!(theme, [SwitchToLight, SwitchToDark]);
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

fn main() {
	let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

	#[cfg(target_os = "macos")]
	let modifiers = Modifiers::SUPER | Modifiers::SHIFT;
	#[cfg(not(target_os = "macos"))]
	let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;

	let hotkey = HotKey::new(Some(modifiers), Code::KeyE);
	hotkey_manager.register(hotkey).expect("Failed to register hotkey");

	let app = Application::new();

	let (tx, rx) = std::sync::mpsc::channel();
	std::thread::spawn(move || {
		let receiver = GlobalHotKeyEvent::receiver();
		loop {
			if let Ok(event) = receiver.recv() {
				if event.state == global_hotkey::HotKeyState::Pressed {
					let _ = tx.send(());
				}
			}
		}
	});

	app.run(|cx: &mut App| {
		theme::init(cx);

		// Set up custom themes directory
		ThemeRegistry::watch_dir(
			"/Users/philocalyst/Projects/emoji-picker/src/themes".into(),
			cx,
			move |_| {
				println!("Themes loaded!");
			},
		)
		.unwrap();

		let registry = ThemeRegistry::global(cx);

		cx.bind_keys([
			KeyBinding::new("cmd-l", JumpToSection, None),
			KeyBinding::new("super-right", SwitchToLight, None),
			KeyBinding::new("super-left", SwitchToDark, None),
		]);

		cx.on_action(|_: &SwitchToLight, cx| {
			let window = cx.global::<AppState>().window;
			window
				.update(cx, |_, window, cx| {
					Theme::change(ThemeMode::Light, Some(window), cx);
				})
				.unwrap();
		});

		cx.on_action(|_: &SwitchToDark, cx| {
			let window = cx.global::<AppState>().window;
			window
				.update(cx, |_, window, cx| {
					Theme::change(ThemeMode::Dark, Some(window), cx);
				})
				.unwrap();
		});

		cx.on_action(|_: &JumpToSection, cx| {
			let state = cx.global::<AppState>();
			let picker_entity = state.picker.clone();
			let window_handle = state.window;

			window_handle
				.update(cx, |_, window, cx| {
					picker_entity.update(cx, |picker, cx| {
						picker.jump_to_section(3, window, cx);
					});
				})
				.unwrap();
		});

		cx.spawn(|ctx: &mut gpui::AsyncApp| {
			let ctx = ctx.clone();
			async move {
				loop {
					if rx.try_recv().is_ok() {
						ctx
							.update(|cx| {
								initialize(cx);
							})
							.expect("Context should be sturdy");
					}
					ctx.background_executor().timer(Duration::from_millis(100)).await;
				}
			}
		})
		.detach();
	});
}

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
			gpui_component::init(cx);

			// No need to call theme::init or watch_dir again - already done in main
			cx.set_global(Theme::default());

			let picker = cx.new(|cx| Picker::new(window, cx));

			window.focus(&picker.read(cx).focus_handle(cx));
			window.activate_window();

			cx.set_global::<AppState>(AppState {
				picker: picker.clone(),
				window: window.window_handle(),
			});

			cx.new(|cx| Root::new(AnyView::from(picker), window, cx))
		},
	)
	.unwrap();
}

fn insert_emoji(emoji: &str) {
	espanso_inject::get_injector(espanso_inject::InjectorCreationOptions::default())
		.and_then(|injector| injector.send_string(emoji, espanso_inject::InjectionOptions::default()));
}
