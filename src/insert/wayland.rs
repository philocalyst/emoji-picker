//! Wayland clipboard management and Hyprland-specific emoji paste insertion.

use std::{thread, time::Duration};
use tracing::{error, warn};

#[cfg(target_os = "linux")]
use crate::integration::linux::{PendingInsertTarget, SHIFT_PASTE_CLASSES};

#[cfg(target_os = "linux")]
pub(crate) fn wl_copy(text: &str) -> std::io::Result<()> {
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
