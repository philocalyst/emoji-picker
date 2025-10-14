use emoji;
use emoji::Emoji;
use emoji_search;
use gpui_component::Root;
use gpui_component::input::InputEvent;
use std::sync::LazyLock;

use gpui::{
    App, Application, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding, Keystroke,
    MouseButton, Pixels, SharedString, Subscription, Window, WindowBounds, WindowOptions, actions,
    black, div, prelude::*, px, rems, rgb, rgba, size, white,
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.set_global(Theme::default());

        let active_text = self.input_state.read(cx).text().clone().to_string();

        let matcher: &'static emoji_search::EmojiSearcher = &*SEARCHER;

        let active_emoji: Vec<&Emoji> = match active_text.as_str() {
            "" => emoji::lookup_by_glyph::iter_emoji().collect(),
            _ => matcher
                .search_best_matching_emojis(&active_text, Some(10))
                .unwrap(),
        };

        div()
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
                TextInput::new(&self.input_state)
                    .appearance(true)
                    .bg(rgb(0xeeeeee))
                    .p(px(4.)),
            )
            .child(
                div()
                    .bg(white())
                    .border_b_1()
                    .border_color(black())
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .justify_center()
                    .gap(rems(0.25))
                    .children(active_emoji.into_iter().enumerate().map(|(id, moji)| {
                        div().id(id).child(moji.glyph).cursor_pointer().on_click(
                            move |_event, _window, _cx| {
                                println!("{moji:?}");
                            },
                        )
                    }))
                    .text_size(rems(1.5)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(300.0), px(300.0)), cx);

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    // Set the theme before creating Root
                    cx.set_global(Theme::default());

                    let input_state =
                        cx.new(|cx| InputState::new(window, cx).placeholder("Type here..."));

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
