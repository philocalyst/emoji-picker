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

use crate::picker::Picker;
use crate::utils::generate_skin_tone_variants;

mod picker;
mod utils;
mod variant_overlay;

actions!(text_input, [Quit,]);

static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
    std::sync::LazyLock::new(|| emoji_search::types::load_emoji_data().unwrap());

static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
    LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));

/// Renders a single emoji button
fn render_emoji_button(emoji_idx: usize, emoji: &Emoji) -> impl IntoElement {
    div()
        .id(emoji_idx)
        .child(emoji.glyph)
        .cursor_pointer()
        .on_click({
            let moji = emoji.to_owned();
            move |_e, _w, _cx| println!("{moji:?}")
        })
}

/// Renders a row of emojis for the virtual list
fn render_emoji_row<'a>(start_idx: usize, end_idx: usize, emojis: &'a [&'a Emoji]) -> Div {
    div()
        .flex()
        .flex_row()
        .children((start_idx..end_idx).map(|emoji_idx| {
            let moji = emojis[emoji_idx];
            render_emoji_button(emoji_idx, moji)
        }))
}

/// Renders the emoji grid with virtual scrolling
fn render_emoji_grid(
    entity: Entity<Picker>,
    emojis: Vec<&'static Emoji>,
    emojis_per_row: usize,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    emoji_text_size: f32,
    scroll_handle: &VirtualListScrollHandle,
) -> impl IntoElement {
    v_virtual_list(
        entity,
        "emojis",
        row_sizes,
        move |_container: &mut Picker, range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|row_idx| {
                    let start_idx = row_idx * emojis_per_row;
                    let end_idx = (start_idx + emojis_per_row).min(emojis.len()) - 1;
                    render_emoji_row(start_idx, end_idx, &emojis)
                })
                .collect()
        },
    )
    .text_size(rems(emoji_text_size))
    .track_scroll(scroll_handle)
    .h_full()
}

fn main() {
    let app = Application::new();

    app.run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(300.0), px(300.0)), cx);

        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                // Set the theme before creating Root
                cx.set_global(Theme::default());
                gpui_component::init(cx);

                let input_state =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Type here..."));

                window.focus(&input_state.read(cx).focus_handle(cx));

                // Subscribe with correct closure signature
                cx.subscribe(
                    &input_state,
                    |_subscriber: Entity<InputState>, _event: &InputEvent, cx: &mut App| {
                        let text = _subscriber.read(cx);
                        eprintln!("Input event: {:?}", text.text());
                    },
                )
                .detach();

                let input_example = cx.new(|cx| Picker {
                    emojis: vec![],
                    scroll_handle: VirtualListScrollHandle::new(),
                    scroll_state: ScrollbarState::default(),
                    input_state: input_state.clone(),
                    recent_keystrokes: vec![],
                    focus_handle: cx.focus_handle(),
                    selected_emoji: Some(5),
                });

                // Wrap InputExample in Root - convert to AnyView
                cx.new(|cx| Root::new(input_example.into(), window, cx))
            },
        )
        .unwrap();

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    });
}
