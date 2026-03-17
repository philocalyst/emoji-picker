//! Cross-platform emoji insertion dispatcher.

mod automated;

#[cfg(target_os = "linux")]
mod unassisted;

#[cfg(target_os = "linux")]
mod wayland;

use std::{thread, time::Duration};

use tracing::debug;
#[cfg(target_os = "linux")]
use tracing::error;


static INSERT_DELAY: Duration = Duration::from_millis(75);

/// How long to keep the process alive after the window closes so the
/// background insertion thread can finish its work.
#[cfg(not(feature = "service"))]
static LINGER_AFTER_CLOSE: Duration = Duration::from_millis(150);

fn insert_emoji(emoji: &str, cx: &gpui::App) {
	let emoji_owned = emoji.to_string();
	debug!(emoji = %emoji, "inserting emoji");

	#[cfg(target_os = "linux")]
	let target = cx.try_global::<crate::integration::linux::PendingInsertTarget>().cloned();

	#[cfg(not(target_os = "linux"))]
	let _ = cx;

	thread::spawn(move || {
		#[cfg(target_os = "macos")]
		{
			thread::sleep(INSERT_DELAY);
			automated::insert_enigo(&emoji_owned);
			return;
		}

		#[cfg(target_os = "linux")]
		{
			use crate::integration::linux::{LinuxSession, detect_linux_session};

			match detect_linux_session() {
				LinuxSession::X11 => {
					thread::sleep(INSERT_DELAY);
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

/// Insert effectively
pub(crate) fn close_and_insert(emoji: &str, cx: &mut gpui::App) {
	// Start the background insertion (types into the now-focused app).
	insert_emoji(emoji, cx);

	// Begin closedown sequence
	cx.shutdown();

	// In service mode the app keeps running, so nothing else to do.
	// In non-service mode we need to quit eventually, but only after
	// the insertion thread has had time to finish.
	#[cfg(not(feature = "service"))]
	{
		cx.spawn(|ctx: &mut gpui::AsyncApp| {
			let ctx = ctx.clone();
			async move {
				ctx.background_executor().timer(LINGER_AFTER_CLOSE).await;

				let _ = ctx.update(|cx| cx.quit());
			}
		})
		.detach();
	}
}
