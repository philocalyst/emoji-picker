use std::{env, sync::LazyLock, thread, time::Duration};

use emoji::Emoji;
use emoji_search;
use enigo::{Enigo, Keyboard, Settings};
#[cfg(target_os = "macos")]
use global_hotkey::hotkey::Modifiers;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, HotKey}};
use gpui::{Action, AnyWindowHandle, App, AppContext, Application, Bounds, Entity, Focusable, Hsla, KeyBinding, Pixels, Size, WindowBounds, WindowKind, WindowOptions, actions, point, px, size};
use gpui_component::{PixelsExt, Root, ThemeColor, theme::{self, Theme, ThemeMode}};
use mouse_position::mouse_position::Mouse;
use nonempty::NonEmpty;
use serde::Deserialize;
use service_manager::*;
use tracing::{Level, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// TODO: This needs finally implement the hold for options logic
// Wanted to borrow inspiration from the mario kart screen, with genders you can move through
// with arrow keys, and have tones in a 3x3?
use crate::picker::Picker;

mod core_row;
mod grouped_grid;
mod listgistics;
mod picker;
mod utilities;
mod variant_overlay;

struct PickerHandle(Entity<Picker>);
impl gpui::Global for PickerHandle {}

macro_rules! bind_keys {
    (
        $cx:ident,
        [ $($static_binding:expr),* $(,)? ],
        jumps: [ $($n:tt),* ]
    ) => {
        $cx.bind_keys([
            $($static_binding,)*
            $(
                KeyBinding::new(
                    concat!("super-", stringify!($n)),
                    JumpToSection { number: $n },
                    None
                ),
            )*
        ]);
    };
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = input, no_json)]
pub struct JumpToSection {
	/// Is confirm with secondary.
	pub number: usize,
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = input, no_json)]
pub struct RotateTones {
	/// Is confirm with secondary.
	pub direction: Direction,
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub enum Direction {
	Forward,
	Backward,
}

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

/// The currently selected emoji
struct SelectedEmoji(Option<NonEmpty<Emoji>>);
impl gpui::Global for SelectedEmoji {}

impl Default for SelectedEmoji {
	fn default() -> Self { Self(None) }
}

/// The tone we're currently on.
struct ToneIndex(u8);
impl gpui::Global for ToneIndex {}

impl ToneIndex {
	const MAX: u8 = 6;

	fn rotate(&mut self, direction: Direction) {
		let place = &mut self.0;

		*place = match direction {
			Direction::Forward => (*place + 1) % Self::MAX,
			Direction::Backward => (*place + Self::MAX - 1) % Self::MAX,
		};
	}
}

impl Default for ToneIndex {
	fn default() -> Self { Self(0) }
}

#[derive(Clone, Copy)]
pub struct PopoverState {
	pub open_emoji: Option<&'static Emoji>,
}

impl Default for PopoverState {
	fn default() -> Self { Self { open_emoji: None } }
}

impl gpui::Global for PopoverState {}

fn main() {
	tracing_subscriber::registry()
		.with(tracing_oslog::OsLogger::new("com.philocalyst.emoji-picker", "default"))
		.with(tracing_subscriber::filter::LevelFilter::INFO)
		.init();

	let args: Vec<String> = env::args().collect();
	if args.contains(&"--service".to_string()) {
		run_app();
		return;
	}

	install_and_start_service();
}

fn install_and_start_service() {
	let label: ServiceLabel = "com.philocalyst.emoji-picker".parse().unwrap();

	// Detect platform native service manager
	let mut manager = <dyn ServiceManager>::native().expect("Failed to detect management platform");

	// Starts service
	if let Err(e) = manager.set_level(ServiceLevel::User) {
		eprintln!("Warning: Could not set service level to User, defaulting to System: {}", e);
	}

	let exe_path = env::current_exe().expect("Failed to get executable path");

	println!("Installing service...");
	match manager.install(ServiceInstallCtx {
		label:             label.clone(),
		program:           exe_path,
		args:              vec!["--service".into()],
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
		cx.set_global::<PopoverState>(PopoverState::default());

		theme::init(cx);

		bind_keys!(
				cx,
				[
						KeyBinding::new("super-q", Quit, None),
						KeyBinding::new("super-w", Quit, None),
						KeyBinding::new("escape", picker::Cancel, None),
						KeyBinding::new("N", RotateTones { direction: Direction::Backward }, None),
						KeyBinding::new("n", RotateTones { direction: Direction::Forward }, None),
						KeyBinding::new("up", picker::MoveUp, None),
						KeyBinding::new("down", picker::MoveDown, None),
						KeyBinding::new("left", picker::MoveLeft, None),
						KeyBinding::new("right", picker::MoveRight, None),
						KeyBinding::new("k", picker::MoveUp, None),
						KeyBinding::new("j", picker::MoveDown, None),
						KeyBinding::new("h", picker::MoveLeft, None),
						KeyBinding::new("l", picker::MoveRight, None),
						KeyBinding::new("space", picker::SelectCurrent, None),
						KeyBinding::new("shift-space", picker::OpenSecondary, None),
						KeyBinding::new("/", picker::FocusSearch, None),
				],
				jumps: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
		);

		// This is exclusively for the coming multi-select option
		cx.on_action(|_: &Quit, cx| {
			// Might not be set???
			// let emojis_to_output = cx.try_global::<SelectedEmoji>();

			// if let Some(emojis) = emojis_to_output {
			// 	insert_emoji(emojis.0.head.glyph);
			// }

			cx.shutdown();
		});

		cx.set_global(Theme {
			colors: ThemeColor {
				// High-level overrides with opacity
				background: Hsla { h: 0.0, s: 0.0, l: 0.05, a: 0.0 },
				foreground: Hsla { h: 0.9, s: 0.3, l: 0.95, a: 0.0 },
				accent: Hsla { h: 0.6, s: 0.7, l: 0.5, a: 0.0 },
				..ThemeColor::default()
			},
			mode: ThemeMode::Dark,
			transparent: Hsla { h: 0.9, s: 0.3, l: 0.95, a: 0.2 },
			..Theme::default()
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

fn get_bounds(
	initial_width: f32,
	initial_height: f32,
	display_size: Size<Pixels>,
	cx: &mut App,
) -> Bounds<Pixels> {
	match Mouse::get_mouse_position() {
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
	}
}

fn initialize(cx: &mut App) {
	let rem_size = 16.0;
	let displays = cx.displays();
	let display = displays.first().expect("no display found");
	let display_size = display.bounds().size;
	let initial_width = (display_size.width.as_f32() * 0.25) + (rem_size * 2.0);
	let initial_height = (display_size.height.as_f32() * 0.4) + (rem_size * 4.0);

	// Get mouse position and calculate window position
	let bounds = get_bounds(initial_width, initial_height, display_size, cx);

	cx.open_window(
		WindowOptions {
			titlebar: None,
			window_background: gpui::WindowBackgroundAppearance::Blurred,
			kind: WindowKind::PopUp,
			window_bounds: Some(WindowBounds::Windowed(bounds)),
			..Default::default()
		},
		|window, cx| {
			gpui_component::init(cx);

			let picker = cx.new(|cx| Picker::new(window, cx));

			window.activate_window();
			window.focus(&picker.read(cx).focus_handle(cx));

			cx.set_global::<AppState>(AppState {
				picker: picker.clone(),
				window: window.window_handle(),
			});

			cx.new(|cx| Root::new(picker, window, cx))
		},
	)
	.unwrap();
}

pub(crate) fn insert_emoji(emoji: &str) {
	let emoji_owned = emoji.to_string();
	thread::spawn(move || {
		// This is TOTALLY a race condition but it's also the BEST solution I have
		thread::sleep(Duration::from_millis(75));
		let mut enigo = Enigo::new(&Settings::default()).unwrap();
		enigo.text(&emoji_owned).unwrap();
	});
}
