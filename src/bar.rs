use gpui::{Entity, IntoElement, ParentElement, Styled, div, img, px, rgb};
use gpui_component::input::{InputState, TextInput};

use crate::utils::get_bar_icons;

/// Renders the emoji category bar
pub(crate) fn render(input_state: &Entity<InputState>) -> impl IntoElement {
	div()
		.flex()
		.flex_row()
		.children(get_bar_icons().iter().map(|path| img(path.to_owned()).w_1().h_1()))
}
