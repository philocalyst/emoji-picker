use gpui::{Entity, IntoElement, Styled, px, rgb};
use gpui_component::input::{Input, InputState};

/// Renders the text input field
pub(crate) fn render(input_state: &Entity<InputState>) -> impl IntoElement {
	Input::new(input_state).appearance(true).bg(rgb(0xeeeeee)).w_full().p(px(4.))
}
