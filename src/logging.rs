//! Platform-aware logging initialization using `tracing`.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) fn init() {
	tracing_subscriber::registry()
		.with(tracing_subscriber::fmt::layer())
		.with(tracing_subscriber::filter::LevelFilter::INFO)
		.init();
}
