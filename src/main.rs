use std::{env, fs, num::{NonZeroI8, NonZeroU8}, sync::LazyLock, thread::{self, sleep}, time::Duration};

use emoji::Emoji;
use emoji_search;
use enigo::{Enigo, Keyboard, Settings};
#[cfg(target_os = "macos")]
use global_hotkey::hotkey::Modifiers;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, HotKey}};
use gpui::{AnyView, AnyWindowHandle, App, Application, Bounds, Entity, Focusable, KeyBinding, WindowBounds, WindowKind, WindowOptions, actions, point, prelude::*, px, size};
use gpui_component::{ActiveTheme, PixelsExt, Root, ThemeRegistry, theme::{self, Theme, ThemeMode}};
use mouse_position::mouse_position::Mouse;
use nonempty::NonEmpty;
use service_manager::*;

use crate::picker::Picker;

mod core_row;
mod grouped_grid;
mod listgistics;
mod picker;
mod utilities;
mod variant_overlay;

struct PickerHandle(Entity<Picker>);
impl gpui::Global for PickerHandle {}

actions!(theme, [SwitchToLight, SwitchToDark]);
actions!(picker, [
	JumpToSection0,
	JumpToSection1,
	JumpToSection2,
	JumpToSection3,
	JumpToSection4,
	JumpToSection5,
	JumpToSection6,
	JumpToSection7,
	JumpToSection8,
	JumpToSection9
]);
actions!(tones, [RotateTonesForward, RotateTonesBackward]);
actions!(text_input, [Quit,]);

static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
	LazyLock::new(|| emoji_search::types::load_emoji_data().unwrap());

static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
	LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));

struct AppState {
	picker: Entity<Picker>,
	window: AnyWindowHandle,
}

impl gpui::Global for AppState {}

/// The currently selected emoji
struct SelectedEmoji(NonEmpty<Emoji>);
impl gpui::Global for SelectedEmoji {}

/// The tone we're currently on.
struct ToneIndex(u8);
impl gpui::Global for ToneIndex {}

impl Default for ToneIndex {
	fn default() -> Self { Self(0) }
}

fn main() {
	// Check if this instance is the service running in background
	let args: Vec<String> = env::args().collect();
	if args.contains(&"--service".to_string()) {
		run_app();
		return;
	}

	// Otherwise, we are the installer/launcher
	install_and_start_service();
}

fn install_and_start_service() {
	let label: ServiceLabel = "com.philocalyst.emoji-picker".parse().unwrap();

	// Detect platform native service manager
	let mut manager = <dyn ServiceManager>::native().expect("Failed to detect management platform");

	// Attempt to use User-level service (LaunchAgent on macOS)
	// This prevents needing sudo and allows GUI interaction
	if let Err(e) = manager.set_level(ServiceLevel::User) {
		eprintln!("Warning: Could not set service level to User, defaulting to System: {}", e);
	}

	let exe_path = env::current_exe().expect("Failed to get executable path");

	println!("Installing service...");
	match manager.install(ServiceInstallCtx {
		label:             label.clone(),
		program:           exe_path,
		args:              vec!["--service".into()], // Crucial: tell the service to run the app logic
		contents:          None,
		username:          None,
		working_directory: None,
		environment:       None,
		autostart:         true,
		restart_policy:    RestartPolicy::Always { delay_secs: Some(5) },
	}) {
		Ok(_) => println!("Service installed successfully."),
		Err(e) => eprintln!("Note: Service install failed (it might already exist): {}", e),
	}

	println!("Starting service...");
	match manager.start(ServiceStartCtx { label }) {
		Ok(_) => println!("Service started successfully in the background."),
		Err(e) => eprintln!("Failed to start service: {}", e),
	}
}

fn run_app() {
	let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

	#[cfg(target_os = "macos")]
	let modifiers = Modifiers::SUPER | Modifiers::SHIFT;
	#[cfg(not(target_os = "macos"))]
	let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;

	let hotkey = HotKey::new(Some(modifiers), Code::KeyE);
	hotkey_manager.register(hotkey).expect("Failed to register hotkey");

	let app = Application::new();

	let (tx, rx) = std::sync::mpsc::channel();
	thread::spawn(move || {
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
		// Set to yellow -- 0
		cx.set_global::<ToneIndex>(ToneIndex(0));

		theme::init(cx);

		// Set up custom themes directory
		ThemeRegistry::watch_dir(
			"/Users/philocalyst/Projects/emoji-picker/src/themes".into(),
			cx,
			move |_| {
				eprintln!("Themes loaded!");
			},
		)
		.unwrap();

		cx.bind_keys([
			KeyBinding::new("super-q", Quit, None),
			KeyBinding::new("super-w", Quit, None),
			KeyBinding::new("escape", Quit, None),
			KeyBinding::new("super-p", RotateTonesBackward, None),
			KeyBinding::new("super-n", RotateTonesForward, None),
			KeyBinding::new("super-0", JumpToSection0, None),
			KeyBinding::new("super-1", JumpToSection1, None),
			KeyBinding::new("super-2", JumpToSection2, None),
			KeyBinding::new("super-3", JumpToSection3, None),
			KeyBinding::new("super-4", JumpToSection4, None),
			KeyBinding::new("super-5", JumpToSection5, None),
			KeyBinding::new("super-6", JumpToSection6, None),
			KeyBinding::new("super-7", JumpToSection7, None),
			KeyBinding::new("super-8", JumpToSection8, None),
			KeyBinding::new("super-9", JumpToSection9, None),
		]);

		// This is exclusively for the coming multi-select option
		cx.on_action(|_: &Quit, cx| {
			// Might not be set???
			let emojis_to_output = cx.try_global::<SelectedEmoji>();

			if let Some(emojis) = emojis_to_output {
				insert_emoji(emojis.0.head.glyph);
			}

			cx.shutdown();
		});

		cx.spawn(|ctx: &mut gpui::AsyncApp| {
			let ctx = ctx.clone();
			async move {
				loop {
					if rx.try_recv().is_ok() {
						ctx
							.update(|cx| {
								// Toggle: if window exists and is valid, focus it
								// Otherwise create a new one
								if let Some(state) = cx.try_global::<AppState>() {
									if state
										.window
										.update(cx, |_, window, _| {
											window.activate_window();
										})
										.is_ok()
									{
										return;
									}
								}
								initialize(cx);
							})
							.expect("Context should be sturdy");
					}
					ctx.background_executor().timer(Duration::from_millis(5)).await;
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

	// Get mouse position and calculate window position
	let bounds = match Mouse::get_mouse_position() {
		Mouse::Position { x, y } => {
			let mut mouse_x = x as f32;
			let mut mouse_y = y as f32;

			// Check right edge overflow
			if mouse_x + initial_width > display_size.width.as_f32() {
				// Position to the left of cursor instead
				mouse_x = mouse_x - initial_width;
				// If still overflows left edge, clamp to left edge
				if mouse_x < 0.0 {
					mouse_x = 0.0;
				}
			}

			// Check bottom edge overflow
			if mouse_y + initial_height > display_size.height.as_f32() {
				// Position above cursor instead
				mouse_y = mouse_y - initial_height;
				// If still overflows top edge, clamp to top edge
				if mouse_y < 0.0 {
					mouse_y = 0.0;
				}
			}

			Bounds::new(point(px(mouse_x), px(mouse_y)), size(px(initial_width), px(initial_height)))
		}
		Mouse::Error => {
			// Fallback to centered if mouse position unavailable
			Bounds::centered(None, size(px(initial_width), px(initial_height)), cx)
		}
	};

	cx.open_window(
		WindowOptions {
			titlebar: None,
			kind: WindowKind::PopUp,
			window_bounds: Some(WindowBounds::Windowed(bounds)),
			..Default::default()
		},
		|window, cx| {
			gpui_component::init(cx);

			cx.set_global(Theme::default());

			let picker = cx.new(|cx| Picker::new(window, cx));

			window.activate_window();
			window.focus(&picker.read(cx).focus_handle(cx));

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
	let emoji_owned = emoji.to_string();
	thread::spawn(move || {
		thread::sleep(Duration::from_millis(50));
		let mut enigo = Enigo::new(&Settings::default()).unwrap();
		enigo.text(&emoji_owned);
	});
}
