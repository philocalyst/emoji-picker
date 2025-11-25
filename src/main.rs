use emoji;
use emoji::Emoji;
use emoji_search;
use gpui_component::input::InputEvent;
use gpui_component::scroll::{Scrollbar, ScrollbarState};
use gpui_component::{Root, VirtualListScrollHandle, v_virtual_list};
use std::rc::Rc;
use std::sync::LazyLock;

use gpui::{
    App, Application, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding, Keystroke,
    Pixels, Size, Window, WindowBounds, WindowOptions, actions, black, div, prelude::*, px, rems,
    rgb, size, white,
};
use gpui_component::input::{InputState, TextInput};
use gpui_component::theme::Theme;

actions!(text_input, [Quit,]);

#[derive(Clone)]
struct InputExample {
    emojis: Vec<Emoji>,
    input_state: Entity<InputState>,
    recent_keystrokes: Vec<Keystroke>,
    focus_handle: FocusHandle,
    /// The position of the selected emoji, if there is one
    selected_emoji: Option<usize>,
    scroll_handle: VirtualListScrollHandle,
    scroll_state: ScrollbarState,
}

impl Focusable for InputExample {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

static EMOJI_DATA: LazyLock<emoji_search::types::EmojiData> =
    std::sync::LazyLock::new(|| emoji_search::types::load_emoji_data().unwrap());

static SEARCHER: LazyLock<emoji_search::EmojiSearcher> =
    LazyLock::new(|| emoji_search::EmojiSearcher::new(&*EMOJI_DATA, None));

impl Render for InputExample {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_text = self.input_state.read(cx).text().clone().to_string();
        let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;
        let active_emoji: Vec<&Emoji> = match active_text.as_str() {
            "" => emoji::lookup_by_glyph::iter_emoji().collect(),
            _ => matcher
                .search_best_matching_emojis(&active_text, Some(1000))
                .unwrap(),
        };

        let emoji_text_size = 1.5;
        let default_emoji_size = w.rem_size() * emoji_text_size;
        let container_width = w.bounds().size.width.to_f64();
        let emojis_per_row = (container_width / default_emoji_size.to_f64()).floor() as usize;

        // Prevent division by zero
        let emojis_per_row = emojis_per_row.max(1);

        // Calculate number of rows needed
        let row_count = (active_emoji.len() + emojis_per_row - 1) / emojis_per_row;

        // Create sizes for rows (not individual emojis)
        let row_sizes: Rc<Vec<Size<Pixels>>> = Rc::new(
            (0..row_count)
                .map(|_| size(container_width.into(), default_emoji_size))
                .collect(),
        );

        div()
            .justify_center()
            .child(
                TextInput::new(&self.input_state)
                    .appearance(true)
                    .cleanable()
                    .bg(rgb(0xeeeeee))
                    .w_full()
                    .p(px(4.)),
            )
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
            .child(
                div()
                    .relative()
                    .child(
                        v_virtual_list(cx.entity().clone(), "emojis", row_sizes, {
                            let emojis = active_emoji.clone();
                            move |container: &mut InputExample,
                                  range: std::ops::Range<usize>,
                                  _window,
                                  cx| {
                                range
                                    .map(|row_idx| {
                                        let start_idx = row_idx * emojis_per_row;
                                        let end_idx =
                                            (start_idx + emojis_per_row).min(emojis.len()) - 1;
                                        div().flex().flex_row().children((start_idx..end_idx).map(
                                            |emoji_idx| {
                                                let moji = &emojis[emoji_idx];
                                                container.selected_emoji = Some(emoji_idx);
                                                div()
                                                    .id(emoji_idx)
                                                    .child(moji.glyph)
                                                    .cursor_pointer()
                                                    .on_click({
                                                        let moji = moji.to_owned();
                                                        move |_e, _w, _cx| println!("{moji:?}")
                                                    })
                                            },
                                        ))
                                    })
                                    .collect()
                            }
                        })
                        .text_size(rems(emoji_text_size))
                        .track_scroll(&self.scroll_handle)
                        .h_full(),
                    )
                    // Overlay for variants
                    .when_some(self.selected_emoji, |parent, emoji_idx| {
                        let emoji = &active_emoji[emoji_idx];
                        parent.child(
                            div()
                                .absolute() // Position absolutely
                                .top_0()
                                .left_0()
                                .w_full()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .p_4()
                                        .rounded_md()
                                        .shadow_lg()
                                        .flex()
                                        .flex_row()
                                        .gap_2(),
                                ),
                        )
                    }),
            )
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
                    |_subscriber: Entity<InputState>, event: &InputEvent, cx: &mut App| {
                        let text = _subscriber.read(cx);
                        eprintln!("Input event: {:?}", text.text());
                    },
                )
                .detach();

                let input_example = cx.new(|cx| InputExample {
                    emojis: vec![],
                    scroll_handle: VirtualListScrollHandle::new(),
                    scroll_state: ScrollbarState::default(),
                    input_state: input_state.clone(),
                    recent_keystrokes: vec![],
                    focus_handle: cx.focus_handle(),
                    selected_emoji: None,
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
