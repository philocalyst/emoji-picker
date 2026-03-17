//! Fallback insertion by copying to clipboard when direct input is unavailable.

#[cfg(target_os = "linux")]
use tracing::{error, warn};

#[cfg(target_os = "linux")]
pub(crate) fn copy_to_clipboard_wayland(emoji: &str) {
	if let Err(e) = super::wayland::wl_copy(emoji) {
		error!("failed to copy emoji to clipboard: {e}");
	} else {
		warn!(
			"non-Hyprland Wayland compositor detected; \
			 emoji copied to clipboard — paste with Ctrl+V"
		);
	}
}
