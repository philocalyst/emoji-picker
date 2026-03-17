//! Picker rendering: the main picker view that composes the list and action handlers.

use gpui::{Context, Edges, Focusable, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, px};
use gpui_component::{StyledExt, gray_800, list::List, purple_400, v_flex};

use crate::components::types::{Picker, ToneIndex};
use crate::keys::*;

impl Render for Picker {
	fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let mut emoji_edges = Edges::all(px(8.));
		emoji_edges.right = px(-4.);

		v_flex()
			.bg(gray_800())
			.text_color(purple_400())
			.on_action(cx.listener(move |_, directive: &RotateTones, _, cx| {
				let current_index = cx.default_global::<ToneIndex>();
				current_index.rotate(directive.direction.clone());
				cx.notify();
			}))
			.on_action(cx.listener(|this, section: &JumpToSection, window, cx| {
				this.jump_to_section(section.number, window, cx);
			}))
			.on_action(cx.listener(|this, _: &MoveUp, window, cx| {
				this.update_selection(window, cx, |d| d.move_up());
			}))
			.on_action(cx.listener(|this, _: &MoveDown, window, cx| {
				this.update_selection(window, cx, |d| d.move_down());
			}))
			.on_action(cx.listener(|this, _: &MoveLeft, window, cx| {
				this.update_selection(window, cx, |d| d.move_left());
			}))
			.on_action(cx.listener(|this, _: &MoveRight, window, cx| {
				this.update_selection(window, cx, |d| d.move_right());
			}))
			.on_action(cx.listener(|this, _: &SelectCurrent, window, cx| {
				this.select_current(window, cx);
			}))
			.on_action(cx.listener(|this, _: &OpenSecondary, window, cx| {
				this.open_secondary(window, cx);
			}))
			.on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
				this.focus_search(window, cx);
			}))
			.on_action(cx.listener(|this, _: &Cancel, window, cx| {
				this.cancel(window, cx);
			}))
			.track_focus(&self.focus_handle(cx))
			.key_context("Picker")
			.size_full()
			.child(List::new(&self.list_state).scrollbar_visible(false).paddings(emoji_edges))
	}
}
