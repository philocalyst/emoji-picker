//! Optional service installation and management, gated behind the `service` feature.

#[cfg(feature = "service")]
use std::env;

#[cfg(feature = "service")]
use service_manager::*;

#[cfg(feature = "service")]
use tracing::{info, error};

#[cfg(feature = "service")]
pub(crate) fn install_and_start_service() {
	let label: ServiceLabel = "com.philocalyst.emoji-picker".parse().unwrap();

	let mut manager = <dyn ServiceManager>::native().expect("failed to detect management platform");

	if let Err(e) = manager.set_level(ServiceLevel::User) {
		error!("could not set service level to User, defaulting to System: {e}");
	}

	let exe_path = env::current_exe().expect("failed to get executable path");

	info!("installing service");
	match manager.install(ServiceInstallCtx {
		label: label.clone(),
		program: exe_path,
		args: vec!["--service".into()],
		contents: None,
		username: None,
		working_directory: None,
		environment: None,
		autostart: true,
		restart_policy: RestartPolicy::Always { delay_secs: Some(5) },
	}) {
		Ok(_) => info!("service installed successfully"),
		Err(e) => error!("service install failed (it might already exist): {e}"),
	}

	info!("starting service");
	match manager.start(ServiceStartCtx { label }) {
		Ok(_) => info!("service started in the background"),
		Err(e) => error!("failed to start service: {e}"),
	}
}
