use std::{env, fs, sync::LazyLock, thread::{self, sleep}, time::Duration};

use emoji_search;
use enigo::{Enigo, Keyboard, Settings};
#[cfg(target_os = "macos")]
use global_hotkey::hotkey::Modifiers;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, HotKey}};
use gpui::{AnyView, AnyWindowHandle, App, Application, Bounds, Entity, Focusable, KeyBinding, WindowBounds, WindowKind, WindowOptions, actions, point, prelude::*, px, size};
use gpui_component::{ActiveTheme, PixelsExt, Root, ThemeRegistry, theme::{self, Theme, ThemeMode}};
use mouse_position::mouse_position::Mouse;
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

actions!(picker, [JumpToSection]);
actions!(theme, [SwitchToLight, SwitchToDark]);
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
			KeyBinding::new("super-right", SwitchToLight, None),
			KeyBinding::new("super-left", SwitchToDark, None),
		]);

		cx.on_action(|_: &Quit, cx| {
			insert_emoji("h");

			cx.shutdown();
		});

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

	// Get mouse position and calculate window position
	let bounds = match Mouse::get_mouse_position() {
		Mouse::Position { x, y } => {
			let mouse_x = x as f32;
			let mouse_y = y as f32;
			let offset = 10.0; // Small offset from cursor

			// Calculate preferred position (right and below cursor)
			let mut window_x = mouse_x + offset;
			let mut window_y = mouse_y + offset;

			// Check right edge overflow
			if window_x + initial_width > display_size.width.as_f32() {
				// Position to the left of cursor instead
				window_x = mouse_x - initial_width - offset;
				// If still overflows left edge, clamp to left edge
				if window_x < 0.0 {
					window_x = 0.0;
				}
			}

			// Check bottom edge overflow
			if window_y + initial_height > display_size.height.as_f32() {
				// Position above cursor instead
				window_y = mouse_y - initial_height - offset;
				// If still overflows top edge, clamp to top edge
				if window_y < 0.0 {
					window_y = 0.0;
				}
			}

			Bounds::new(point(px(window_x), px(window_y)), size(px(initial_width), px(initial_height)))
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
	let emoji_owned = emoji.to_string();
	thread::spawn(move || {
		thread::sleep(Duration::from_millis(50));
		let mut enigo = Enigo::new(&Settings::default()).unwrap();
		enigo.text(&emoji_owned);
	});
}
