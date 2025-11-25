use gpui::Styled;
use gpui::{Entity, IntoElement, px, rgb};
use gpui_component::input::{InputState, TextInput};

/// Renders the text input field
pub(crate) fn render(input_state: &Entity<InputState>) -> impl IntoElement {
    TextInput::new(input_state)
        .appearance(true)
        .cleanable()
        .bg(rgb(0xeeeeee))
        .w_full()
        .p(px(4.))
}
