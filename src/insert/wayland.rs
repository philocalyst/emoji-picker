//! Wayland clipboard management and Hyprland-specific emoji paste insertion.
#[cfg(target_os = "linux")]
use crate::integration::linux::{PendingInsertTarget, SHIFT_PASTE_CLASSES};
use std::{thread, time::Duration};
use tracing::{error, warn};

#[cfg(target_os = "linux")]
pub(crate) fn wl_copy(text: &str) -> std::io::Result<()> {
	use wl_clipboard_rs::copy::{MimeType, Options, Source};
	let opts = Options::new();
	opts
		.copy(
			Source::Bytes(text.as_bytes().to_vec().into()),
			MimeType::Specific("text/plain".to_string()),
		)
		.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

#[cfg(target_os = "linux")]
fn wl_paste() -> std::io::Result<Option<String>> {
	use std::io::Read;
	use wl_clipboard_rs::paste::{ClipboardType, Error, MimeType, Seat, get_contents};

	let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);
	match result {
		Ok((mut pipe, _)) => {
			let mut contents = vec![];
			pipe.read_to_end(&mut contents)?;
			let text = String::from_utf8_lossy(&contents).into_owned();
			if text.trim().is_empty() { Ok(None) } else { Ok(Some(text)) }
		}
		Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => Ok(None),
		Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
	}
}

#[cfg(target_os = "linux")]
pub(crate) fn insert_hyprland(emoji: &str, target: Option<&PendingInsertTarget>) {
	use std::process::Command;

	let Some(target) = target else {
		warn!("no Hyprland insert target captured; falling back to clipboard copy");
		let _ = wl_copy(emoji);
		return;
	};

	let Some(address) = target.hyprland_address.as_deref() else {
		warn!("no Hyprland window address; falling back to clipboard copy");
		let _ = wl_copy(emoji);
		return;
	};

	let original_clipboard = wl_paste().ok().flatten();

	if let Err(e) = wl_copy(emoji) {
		error!("wl-copy failed: {e}");
		return;
	}

	thread::sleep(Duration::from_millis(25));

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

	thread::sleep(Duration::from_millis(100));

	if let Some(original) = original_clipboard {
		if let Err(e) = wl_copy(&original) {
			warn!("failed to restore clipboard: {e}");
		}
	}
}
