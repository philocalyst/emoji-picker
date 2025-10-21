use emoji;
use emoji::Emoji;
use emoji_search;
use gpui_component::input::InputEvent;
use gpui_component::{Root, h_virtual_list};
use std::rc::Rc;
use std::sync::LazyLock;

use gpui::{
    App, Application, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding, Keystroke,
    Pixels, Size, Window, WindowBounds, WindowOptions, actions, black, div, prelude::*, px, rems,
    rgb, size, uniform_list, white,
};
use gpui_component::input::{InputState, TextInput};
use gpui_component::theme::Theme;

actions!(text_input, [Quit,]);

#[derive(Clone)]
struct InputExample {
    emojis: Vec<Emoji>,
    emoji_size: Size<Pixels>,
    input_state: Entity<InputState>,
    recent_keystrokes: Vec<Keystroke>,
    focus_handle: FocusHandle,
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_text = self.input_state.read(cx).text().clone().to_string();

        let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;

        let active_emoji: Vec<&Emoji> = match active_text.as_str() {
            "" => emoji::lookup_by_glyph::iter_emoji().collect(),
            _ => matcher
                .search_best_matching_emojis(&active_text, Some(10))
                .unwrap(),
        };

        // Create a list for the sizes of all the emoji, which is going to be largely boilerplate because they're monospaced
        let emoji_sizes: Rc<Vec<Size<Pixels>>> =
            Rc::new(active_emoji.iter().map(|_| self.emoji_size).collect());

        div()
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
                h_virtual_list(cx.entity().clone(), "emojis", emoji_sizes, {
                    let emojis = active_emoji.clone();
                    move |_this: &mut InputExample, range: std::ops::Range<usize>, _window, cx| {
                        range
                            .map(|idx| {
                                let moji = &emojis[idx];
                                div().id(idx).child(moji.glyph).cursor_pointer().on_click({
                                    let moji = moji.to_owned();
                                    move |_e, _w, _cx| println!("{moji:?}")
                                })
                            })
                            .collect()
                    }
                })
                .flex()
                .flex_wrap()
                .h_full()
                .text_size(rems(1.5)),
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

                // We want the emoji size to be a function of the text size, so as a user scales the window, more emojis will populate.
                let rems = window.rem_size() * 2;
                let emoji_size = size(rems, rems);

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
                    emoji_size,
                    input_state: input_state.clone(),
                    recent_keystrokes: vec![],
                    focus_handle: cx.focus_handle(),
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
