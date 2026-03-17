//! macOS accessory policy to hide the app from the Dock.

use tracing::debug;

/// Sets the application activation policy to Accessory so it doesn't appear in
/// the Dock. TODO: Help this get merged into GPUI-CE (https://github.com/zed-industries/zed/pull/43822)
pub(crate) fn set_accessory_policy() {
	debug!("setting macOS activation policy to Accessory");
	unsafe {
		use objc2::{MainThreadMarker, msg_send};
		use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
		let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
		let _: () = msg_send![&*app, setActivationPolicy: NSApplicationActivationPolicy::Accessory];
	}
}
