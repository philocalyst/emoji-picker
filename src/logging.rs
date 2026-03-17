//! Platform-aware logging initialization using `tracing`.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) fn init() {
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
}
