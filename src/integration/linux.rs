//! Linux session detection, Hyprland support, and terminal paste class
//! handling.

use tracing::warn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinuxSession {
	WaylandHyprland,
	WaylandOther,
	X11,
	Unknown,
}

pub(crate) fn detect_linux_session() -> LinuxSession {
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
	pub hyprland_address: Option<String>,
	pub class:            Option<String>,
}

impl gpui::Global for PendingInsertTarget {}

/// Terminal window classes that need Ctrl+Shift+V instead of Ctrl+V.
pub(crate) const SHIFT_PASTE_CLASSES: &[&str] = &[
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

pub(crate) fn capture_hyprland_active_window() -> PendingInsertTarget {
	use std::process::Command;

	let output = match Command::new("hyprctl").args(["activewindow", "-j"]).output() {
		Ok(o) => o,
		Err(e) => {
			warn!("failed to run hyprctl activewindow: {e}");
			return PendingInsertTarget::default();
		}
	};

	let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
		Ok(v) => v,
		Err(e) => {
			warn!("failed to parse hyprctl activewindow JSON: {e}");
			return PendingInsertTarget::default();
		}
	};

	PendingInsertTarget {
		hyprland_address: json["address"].as_str().map(String::from),
		class:            json["class"].as_str().map(String::from),
	}
}
