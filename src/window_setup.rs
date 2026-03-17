//! Window positioning logic with smart bound detection based on cursor
//! position.
use enigo::{Enigo, Mouse, Settings};
use gpui::{App, Bounds, Pixels, Size, point, px, size};
use gpui_component::PixelsExt;
use tracing::{debug, warn};

pub(crate) fn get_bounds(
	initial_width: f32,
	initial_height: f32,
	display_size: Size<Pixels>,
	cx: &mut App,
) -> Bounds<Pixels> {
	let enigo = match Enigo::new(&Settings::default()) {
		Ok(e) => e,
		Err(err) => {
			warn!(%err, "failed to initialize enigo, centering window");
			return Bounds::centered(None, size(px(initial_width), px(initial_height)), cx);
		}
	};

	match enigo.location() {
		Ok((x, y)) => {
			let mut mouse_x = x as f32;
			let mut mouse_y = y as f32;
			debug!(mouse_x, mouse_y, "cursor position for window placement");

			if mouse_x + initial_width > display_size.width.as_f32() {
				mouse_x = (mouse_x - initial_width).max(0.0);
			}
			if mouse_y + initial_height > display_size.height.as_f32() {
				mouse_y = (mouse_y - initial_height).max(0.0);
			}

			Bounds::new(point(px(mouse_x), px(mouse_y)), size(px(initial_width), px(initial_height)))
		}
		Err(err) => {
			warn!(%err, "mouse position unavailable, centering window");
			Bounds::centered(None, size(px(initial_width), px(initial_height)), cx)
		}
	}
}
