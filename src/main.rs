//! B.O.B (BOB prOffers flamBoyance): a fast, keyboard-driven emoji picker.

mod components;
mod emoji_sizing;
mod insert;
mod integration;
mod keys;
mod lifecycle;
mod logging;
#[cfg(feature = "service")]
mod service;
mod window_setup;

fn main() {
	logging::init();

	#[cfg(feature = "service")]
	{
		let args: Vec<String> = std::env::args().collect();
		if args.contains(&"--service".to_string()) {
			lifecycle::run_app();
			return;
		}

		service::install_and_start_service();
		return;
	}

	#[cfg(not(feature = "service"))]
	lifecycle::run_app();
}
