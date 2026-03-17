//! Application lifecycle management for both service and non-service
//! configurations.

#[cfg(feature = "service")]
use std::thread;
#[cfg(feature = "service")]
use std::time::Duration;

#[cfg(feature = "service")]
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, HotKey, Modifiers}};
use gpui::{AnyWindowHandle, App, AppContext, Application, Entity, Hsla, WindowBounds, WindowKind, WindowOptions};
use gpui_component::{PixelsExt, Root, ThemeColor, theme::{self, Theme, ThemeMode}};
use tracing::{debug, info};

use crate::{components::{Picker, types::{PopoverState, ToneIndex}}, keys::{self, Quit}, window_setup};

#[allow(dead_code)]
pub(crate) struct AppState {
	pub picker: Entity<Picker>,
	pub window: AnyWindowHandle,
}

impl gpui::Global for AppState {}

pub(crate) fn run_app() {
	info!("starting app");

	#[cfg(feature = "service")]
	let hotkey_manager = GlobalHotKeyManager::new().expect("failed to create hotkey manager");

	#[cfg(feature = "service")]
	{
		#[cfg(target_os = "macos")]
		let modifiers = Modifiers::SUPER | Modifiers::SHIFT;
		#[cfg(not(target_os = "macos"))]
		let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;

		let hotkey = HotKey::new(Some(modifiers), Code::KeyE);
		hotkey_manager.register(hotkey).expect("failed to register hotkey");
	}

	let app = Application::new();

	#[cfg(feature = "service")]
	let (tx, rx) = std::sync::mpsc::channel();

	#[cfg(feature = "service")]
	thread::spawn(move || {
		let receiver = GlobalHotKeyEvent::receiver();
		loop {
			if let Ok(event) = receiver.recv() {
				if event.state == global_hotkey::HotKeyState::Pressed {
					debug!("hotkey pressed; toggling picker");
					let _ = tx.send(());
				}
			}
		}
	});

	app.run(|cx: &mut App| {
		cx.set_global::<ToneIndex>(ToneIndex(0));
		cx.set_global::<PopoverState>(PopoverState::default());

		#[cfg(target_os = "macos")]
		crate::integration::macos::set_accessory_policy();

		theme::init(cx);
		keys::bind_all(cx);

		cx.on_action(|_: &Quit, cx| {
			cx.shutdown();
		});

		cx.set_global(Theme {
			colors: ThemeColor {
				background: Hsla { h: 0.0, s: 0.0, l: 0.05, a: 0.0 },
				foreground: Hsla { h: 0.9, s: 0.3, l: 0.95, a: 0.0 },
				accent: Hsla { h: 0.6, s: 0.7, l: 0.5, a: 0.0 },
				..ThemeColor::default()
			},
			mode: ThemeMode::Dark,
			transparent: Hsla { h: 0.9, s: 0.3, l: 0.95, a: 0.2 },
			..Theme::default()
		});

		#[cfg(feature = "service")]
		cx.spawn(|ctx: &mut gpui::AsyncApp| {
			let ctx = ctx.clone();
			async move {
				loop {
					if rx.try_recv().is_ok() {
						ctx
							.update(|cx| {
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

								#[cfg(target_os = "linux")]
								{
									use crate::integration::linux::{
										LinuxSession, PendingInsertTarget, capture_hyprland_active_window,
										detect_linux_session,
									};
									if detect_linux_session() == LinuxSession::WaylandHyprland {
										let target = capture_hyprland_active_window();
										cx.set_global::<PendingInsertTarget>(target);
									}
								}

								initialize(cx);
							})
							.expect("context should be available");
					}
					ctx.background_executor().timer(Duration::from_millis(5)).await;
				}
			}
		})
		.detach();

		#[cfg(not(feature = "service"))]
		{
			#[cfg(target_os = "linux")]
			{
				use crate::integration::linux::{
					LinuxSession, PendingInsertTarget, capture_hyprland_active_window, detect_linux_session,
				};
				if detect_linux_session() == LinuxSession::WaylandHyprland {
					let target = capture_hyprland_active_window();
					cx.set_global::<PendingInsertTarget>(target);
				}
			}

			initialize(cx);
		}
	});
}

fn initialize(cx: &mut App) {
	let rem_size = 16.0;
	let displays = cx.displays();
	let display = displays.first().expect("no display found");
	let display_size = display.bounds().size;
	let initial_width = (display_size.width.as_f32() * 0.25) + (rem_size * 2.0);
	let initial_height = (display_size.height.as_f32() * 0.4) + (rem_size * 4.0);

	let bounds = window_setup::get_bounds(initial_width, initial_height, display_size, cx);
	debug!(?bounds, "opening picker window");

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
