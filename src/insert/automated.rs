//! Ideal emoji insertion via Enigo (macOS and X11).

#[cfg(not(target_os = "linux"))]
use enigo::{Enigo, Keyboard, Settings};

pub(crate) fn insert_enigo(emoji: &str) {
	#[cfg(not(target_os = "linux"))]
	{
		use tracing::info;

		let mut enigo = Enigo::new(&Settings::default()).unwrap();
		enigo.text(emoji).unwrap();
	}

	#[cfg(target_os = "linux")]
	{
		let mut enigo = enigo::Enigo::new(&enigo::Settings::default()).unwrap();
		enigo::Keyboard::text(&mut enigo, emoji).unwrap();
	}
}
