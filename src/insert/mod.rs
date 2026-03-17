//! Cross-platform emoji insertion dispatcher.

mod automated;
mod unassisted;

#[cfg(target_os = "linux")]
mod wayland;

use std::{thread, time::Duration};
use tracing::debug;
#[cfg(target_os = "linux")]
use tracing::error;

pub(crate) fn insert_emoji(emoji: &str, cx: &gpui::App) {
	let emoji_owned = emoji.to_string();
	debug!(emoji = %emoji, "inserting emoji");

	#[cfg(target_os = "linux")]
	let target = cx
		.try_global::<crate::integration::linux::PendingInsertTarget>()
		.cloned();

	#[cfg(not(target_os = "linux"))]
	let _ = cx;

	thread::spawn(move || {
		#[cfg(target_os = "macos")]
		{
			thread::sleep(Duration::from_millis(75));
			automated::insert_enigo(&emoji_owned);
			return;
		}

		#[cfg(target_os = "linux")]
		{
			use crate::integration::linux::{LinuxSession, detect_linux_session};

			match detect_linux_session() {
				LinuxSession::X11 => {
					thread::sleep(Duration::from_millis(75));
					automated::insert_enigo(&emoji_owned);
				}
				LinuxSession::WaylandHyprland => {
					wayland::insert_hyprland(&emoji_owned, target.as_ref());
				}
				LinuxSession::WaylandOther => {
					unassisted::copy_to_clipboard_wayland(&emoji_owned);
				}
				LinuxSession::Unknown => {
					error!("could not detect display session; emoji not inserted");
				}
			}
		}
	});
}
