use gpui::{Entity, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled, div, img, px, rgb};
use gpui_component::{VirtualListScrollHandle, gray_300, input::{InputState, TextInput}, scroll::Scrollbar};

use crate::utils::get_bar_icons;

/// Renders the emoji category bar
pub(crate) fn render(scrollbar: VirtualListScrollHandle) -> impl IntoElement {
	div()
		.id(60)
		.absolute()
		.top_0()
		.left_0()
		.bg(gray_300())
		.opacity(10f32)
		.on_click(move |_event, _other, _ctx| {
			scrollbar.scroll_to_bottom();
		})
		.flex()
		.flex_row()
		.justify_center()
		.children(get_bar_icons().iter().map(|path| img(path.to_owned()).w_8().h_8()))
}
