use std::{env, sync::LazyLock, thread, time::Duration};

use emoji::Emoji;
use emoji_search;
#[cfg(not(target_os = "linux"))]
use enigo::{Enigo, Keyboard, Settings};
use global_hotkey::{
	GlobalHotKeyEvent, GlobalHotKeyManager,
	hotkey::{Code, HotKey, Modifiers},
};
use gpui::{
	Action, AnyWindowHandle, App, AppContext, Application, Bounds, Entity, Hsla, KeyBinding, Pixels,
	Size, WindowBounds, WindowKind, WindowOptions, actions, point, px, size,
};
use gpui_component::{
	PixelsExt, Root, ThemeColor,
	theme::{self, Theme, ThemeMode},
};
use mouse_position::mouse_position::Mouse;
use nonempty::NonEmpty;
use serde::Deserialize;
use service_manager::*;
#[cfg(target_os = "linux")]
use tracing::{error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

use Direction::{Backward, Forward};

actions!(theme, [SwitchToLight, SwitchToDark]);
actions!(all, [Quit, Cancel]);

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
	fn default() -> Self {
		Self(None)
	}
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
	fn default() -> Self {
		Self(0)
	}
}

// -- Wayland / Hyprland support ---------------------------------------------------

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxSession {
	WaylandHyprland,
	WaylandOther,
	X11,
	Unknown,
}

#[cfg(target_os = "linux")]
fn detect_linux_session() -> LinuxSession {
	let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
		|| std::env::var("XDG_SESSION_TYPE").ok().as_deref() == Some("wayland");
	if wayland {
		if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
			LinuxSession::WaylandHyprland
		} else {
			LinuxSession::WaylandOther
		}
	} else if std::env::var_os("DISPLAY").is_some() {
		LinuxSession::X11
	} else {
		LinuxSession::Unknown
	}
}

/// Stores the window that was focused before the picker opened.
/// Captured via `hyprctl activewindow -j` on Hyprland/Wayland.
#[derive(Clone, Debug, Default)]
pub(crate) struct PendingInsertTarget {
	/// Hyprland window address (e.g. "0x5678abcd")
	pub hyprland_address: Option<String>,
	/// Window class for terminal-paste detection
	pub class: Option<String>,
}

impl gpui::Global for PendingInsertTarget {}

/// Terminal window classes that need Ctrl+Shift+V instead of Ctrl+V.
#[cfg(target_os = "linux")]
const SHIFT_PASTE_CLASSES: &[&str] = &[
	"kitty",
	"alacritty",
	"foot",
	"wezterm",
	"terminator",
	"tilix",
	"gnome-terminal",
	"konsole",
	"xterm",
	"urxvt",
	"st",
	"rio",
	"ghostty",
];

#[cfg(target_os = "linux")]
fn capture_hyprland_active_window() -> PendingInsertTarget {
	use std::process::Command;

	let output = match Command::new("hyprctl").args(["activewindow", "-j"]).output() {
		Ok(o) => o,
		Err(e) => {
			warn!("Failed to run hyprctl activewindow: {e}");
			return PendingInsertTarget::default();
		}
	};

	let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
		Ok(v) => v,
		Err(e) => {
			warn!("Failed to parse hyprctl activewindow JSON: {e}");
			return PendingInsertTarget::default();
		}
	};

	PendingInsertTarget {
		hyprland_address: json["address"].as_str().map(String::from),
		class: json["class"].as_str().map(String::from),
	}
}

#[derive(Clone, Copy)]
pub struct PopoverState {
	pub open_emoji: Option<&'static Emoji>,
}

impl Default for PopoverState {
	fn default() -> Self {
		Self { open_emoji: None }
	}
}

impl gpui::Global for PopoverState {}

fn main() {
	#[cfg(target_os = "macos")]
	tracing_subscriber::registry()
		.with(tracing_oslog::OsLogger::new("com.philocalyst.emoji-picker", "default"))
		.with(tracing_subscriber::filter::LevelFilter::INFO)
		.init();

	#[cfg(not(target_os = "macos"))]
	tracing_subscriber::registry()
		.with(tracing_subscriber::fmt::layer())
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
		label: label.clone(),
		program: exe_path,
		args: vec!["--service".into()],
		contents: None,
		username: None,
		working_directory: None,
		environment: None,
		autostart: true,
		restart_policy: RestartPolicy::Always { delay_secs: Some(5) },
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

		// Important that it doesn't show within the dock,
		// TODO: Help this get merged into GPUI-CE so this isn't needed (https://github.com/zed-industries/zed/pull/43822)
		#[cfg(target_os = "macos")]
		unsafe {
			use objc2::{MainThreadMarker, msg_send};
			use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
			let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
			let _: () = msg_send![&*app, setActivationPolicy: NSApplicationActivationPolicy::Accessory];
		}

		theme::init(cx);

		bind_keys!(
				cx,
				[
					// Global bindings
					KeyBinding::new("super-q", Quit, None),
					KeyBinding::new("super-w", Quit, None),
					KeyBinding::new("escape", Cancel, None),
					KeyBinding::new("enter", Cancel, None),

					// Picker-scoped bindings
					KeyBinding::new("up", picker::MoveUp, Some("List")),
					KeyBinding::new("down", picker::MoveDown, Some("List")),
					KeyBinding::new("left", picker::MoveLeft, Some("List")),
					KeyBinding::new("right", picker::MoveRight, Some("List")),
					KeyBinding::new(",", picker::OpenSecondary, Some("List")),
					KeyBinding::new("shift-space", picker::SelectCurrent, Some("List")),

					KeyBinding::new("space", picker::SelectCurrent, Some("ListBody")),
					KeyBinding::new("N", RotateTones { direction: Backward }, Some("ListBody")),
					KeyBinding::new("n", RotateTones { direction: Forward }, Some("ListBody")),
					KeyBinding::new("k", picker::MoveUp, Some("ListBody")),
					KeyBinding::new("j", picker::MoveDown, Some("ListBody")),
					KeyBinding::new("h", picker::MoveLeft, Some("ListBody")),
					KeyBinding::new("l", picker::MoveRight, Some("ListBody")),
					KeyBinding::new("/", picker::FocusSearch, Some("ListBody")),

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

								// Capture the currently-focused window BEFORE we
								// open the picker (Wayland/Hyprland).
								#[cfg(target_os = "linux")]
								if detect_linux_session() == LinuxSession::WaylandHyprland {
									let target = capture_hyprland_active_window();
									cx.set_global::<PendingInsertTarget>(target);
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
			let list_state = picker.read(cx).list_state.clone();
			list_state.update(cx, |list, cx| list.focus(window, cx));

			cx.set_global::<AppState>(AppState {
				picker: picker.clone(),
				window: window.window_handle(),
			});

			cx.new(|cx| Root::new(picker, window, cx))
		},
	)
	.unwrap();
}

pub(crate) fn insert_emoji(emoji: &str, cx: &App) {
	let emoji_owned = emoji.to_string();

	#[cfg(target_os = "linux")]
	let target = cx.try_global::<PendingInsertTarget>().cloned();

	// Suppress unused-variable warning on non-Linux
	#[cfg(not(target_os = "linux"))]
	let _ = cx;

	thread::spawn(move || {
		#[cfg(target_os = "macos")]
		{
			thread::sleep(Duration::from_millis(75));
			let mut enigo = Enigo::new(&Settings::default()).unwrap();
			enigo.text(&emoji_owned).unwrap();
			return;
		}

		#[cfg(target_os = "linux")]
		{
			match detect_linux_session() {
				LinuxSession::X11 => {
					thread::sleep(Duration::from_millis(75));
					let mut enigo = enigo::Enigo::new(&enigo::Settings::default()).unwrap();
					enigo::Keyboard::text(&mut enigo, &emoji_owned).unwrap();
				}
				LinuxSession::WaylandHyprland => {
					insert_emoji_wayland_hyprland(&emoji_owned, target.as_ref());
				}
				LinuxSession::WaylandOther => {
					// Best-effort: just copy to clipboard, user can paste manually
					if let Err(e) = wl_copy(&emoji_owned) {
						error!("Failed to copy emoji to clipboard: {e}");
					} else {
						warn!(
							"Non-Hyprland Wayland compositor detected. \
							 Emoji copied to clipboard — paste with Ctrl+V."
						);
					}
				}
				LinuxSession::Unknown => {
					error!("Could not detect display session; emoji not inserted.");
				}
			}
		}
	});
}

#[cfg(target_os = "linux")]
fn wl_copy(text: &str) -> std::io::Result<()> {
	use std::io::Write;
	use std::process::{Command, Stdio};

	let mut child =
		Command::new("wl-copy").arg("--type").arg("text/plain").stdin(Stdio::piped()).spawn()?;

	if let Some(mut stdin) = child.stdin.take() {
		stdin.write_all(text.as_bytes())?;
	}
	child.wait()?;
	Ok(())
}

#[cfg(target_os = "linux")]
fn wl_paste() -> std::io::Result<Option<String>> {
	use std::process::Command;

	let output = Command::new("wl-paste").arg("--no-newline").output()?;

	let stdout = String::from_utf8_lossy(&output.stdout);
	if stdout.contains("Nothing is copied") || stdout.trim().is_empty() {
		return Ok(None);
	}
	Ok(Some(stdout.into_owned()))
}

#[cfg(target_os = "linux")]
fn insert_emoji_wayland_hyprland(emoji: &str, target: Option<&PendingInsertTarget>) {
	use std::process::Command;

	let Some(target) = target else {
		warn!("No Hyprland insert target captured; falling back to clipboard copy");
		let _ = wl_copy(emoji);
		return;
	};

	let Some(address) = target.hyprland_address.as_deref() else {
		warn!("No Hyprland window address; falling back to clipboard copy");
		let _ = wl_copy(emoji);
		return;
	};

	// 1. Save original clipboard contents
	let original_clipboard = wl_paste().ok().flatten();

	// 2. Copy emoji to clipboard
	if let Err(e) = wl_copy(emoji) {
		error!("wl-copy failed: {e}");
		return;
	}

	// Small delay to let wl-copy settle
	thread::sleep(Duration::from_millis(25));

	// 3. Send paste shortcut to the original window via hyprctl
	let needs_shift = target
		.class
		.as_deref()
		.map(|c| {
			let lower = c.to_lowercase();
			SHIFT_PASTE_CLASSES.iter().any(|t| lower.contains(t))
		})
		.unwrap_or(false);

	let shortcut = if needs_shift {
		format!("CONTROL SHIFT, V, address:{address}")
	} else {
		format!("CONTROL, V, address:{address}")
	};

	let result = Command::new("hyprctl").args(["dispatch", "sendshortcut", &shortcut]).output();

	if let Err(e) = result {
		error!("hyprctl dispatch sendshortcut failed: {e}");
	}

	// 4. Wait for paste to complete
	thread::sleep(Duration::from_millis(100));

	// 5. Restore original clipboard
	if let Some(original) = original_clipboard {
		if let Err(e) = wl_copy(&original) {
			warn!("Failed to restore clipboard: {e}");
		}
	}
}
