use emoji;
use emoji::Emoji;
use emoji_search;
use gpui_component::input::InputEvent;
use gpui_component::scroll::{Scrollbar, ScrollbarState};
use gpui_component::{Root, VirtualListScrollHandle, v_virtual_list};
use std::rc::Rc;
use std::sync::LazyLock;

use gpui::{
    App, Application, Bounds, Context, Div, Entity, FocusHandle, Focusable, KeyBinding, Keystroke,
    Pixels, Size, Window, WindowBounds, WindowOptions, actions, black, div, prelude::*, px, rems,
    rgb, size, white,
};
use gpui_component::input::{InputState, TextInput};
use gpui_component::theme::Theme;

use crate::utils::{calculate_emojis_per_row, generate_row_sizes, search_emojis};
use crate::{input, variant_overlay};

#[derive(Clone)]
pub(crate) struct Picker {
    pub(crate) emojis: Vec<Emoji>,
    pub(crate) input_state: Entity<InputState>,
    pub(crate) recent_keystrokes: Vec<Keystroke>,
    pub(crate) focus_handle: FocusHandle,
    /// The position of the selected emoji, if there is one
    pub(crate) selected_emoji: Option<usize>,
    pub(crate) scroll_handle: VirtualListScrollHandle,
    pub(crate) scroll_state: ScrollbarState,
}

impl Focusable for Picker {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Picker {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_text = self.input_state.read(cx).text().clone().to_string();
        let active_emoji = search_emojis(&active_text);

        let emoji_text_size = 1.5;
        let default_emoji_size = w.rem_size() * emoji_text_size;
        let container_width = w.bounds().size.width.to_f64();
        let emojis_per_row = calculate_emojis_per_row(container_width, default_emoji_size);
        let row_sizes = generate_row_sizes(
            active_emoji.len(),
            emojis_per_row,
            container_width,
            default_emoji_size,
        );

        div()
            .justify_center()
            .child(input::render(&self.input_state))
            .bg(rgb(0xaaaaaa))
            .track_focus(&self.focus_handle(cx))
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .bg(white())
                    .border_b_1()
                    .border_color(black())
                    .flex()
                    .flex_row()
                    .justify_between(),
            )
            .child(div().size_full().child(Self::render_grid(
                cx.entity().clone(),
                active_emoji.clone(),
                emojis_per_row,
                self.selected_emoji,
                row_sizes,
                emoji_text_size,
                &self.scroll_handle,
            )))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .child(Scrollbar::vertical(&self.scroll_state, &self.scroll_handle)),
            )
    }
}
