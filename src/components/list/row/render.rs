//! Row rendering: each emoji cell with selection highlight, tooltips, and
//! popover support.

use gpui::{
	App, BorrowAppContext, BoxShadow, Edges, Hsla, InteractiveElement, IntoElement, MouseButton,
	ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
	hsla, px,
};
use gpui_component::{StyledExt, h_flex, popover::Popover, tooltip::Tooltip};

use super::types::{EmojiRow, EmojiWrapper};
use crate::{
	components::{
		types::{PopoverState, ToneIndex},
		variants,
	},
	insert::close_and_insert,
};

impl RenderOnce for EmojiRow {
	fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
		let between_row_padding = Edges { top: px(5.), bottom: px(5.), ..Default::default() };
		let emoji_centering = Edges { left: px(1.), right: px(-1.), ..Default::default() };
		let font_size = self.font_size;
		let selected_row = self.contains_selection;
		let selected_col = self.selected_column;

		h_flex()
			.key_context("ListBody")
			.track_focus(&self.body_focus_handle)
			.paddings(between_row_padding)
			.gap_2()
			.children(self.emojis.into_iter().enumerate().map(move |(idx, emoji)| {
				let tone_index = cx.global::<ToneIndex>();
				let is_selected = selected_row && selected_col == Some(idx);

				let pure_emoji = if let Some(tones) = emoji.skin_tones {
					tones.get(tone_index.0 as usize).unwrap_or(emoji).glyph
				} else {
					emoji.glyph
				};

				let mut base_element = div()
					.bg(Hsla { h: 0., s: 0., l: 1., a: 0.1 })
					.text_size(font_size)
					.paddings(emoji_centering)
					.id(pure_emoji)
					.shadow(vec![
						BoxShadow {
							color: hsla(0.0, 0.0, 0.0, 0.25),
							offset: gpui::point(px(0.), px(1.)),
							blur_radius: px(2.),
							spread_radius: px(0.),
						},
						BoxShadow {
							color: hsla(0.0, 0.0, 0.0, 0.15),
							offset: gpui::point(px(0.), px(8.)),
							blur_radius: px(16.),
							spread_radius: px(-2.),
						},
					]);

				if is_selected {
					base_element = base_element.bg(hsla(0.0, 0.0, 1.0, 0.2)).shadow(vec![BoxShadow {
						color: hsla(0.78, 0.6, 0.5, 0.8),
						offset: gpui::point(gpui::px(0.), gpui::px(4.)),
						blur_radius: gpui::px(12.),
						spread_radius: gpui::px(7.),
					}]);
				}

				let base_element = base_element
					.hover(move |s: StyleRefinement| {
						s.shadow(vec![BoxShadow {
							color: hsla(0.78, 0.6, 0.5, 0.8),
							offset: gpui::point(gpui::px(0.), gpui::px(4.)),
							blur_radius: gpui::px(12.),
							spread_radius: gpui::px(7.),
						}])
					})
					.tooltip(move |window, cx| Tooltip::new(emoji.name).build(window, cx))
					.corner_radii(gpui::Corners::all(px(5f32)))
					.cursor_pointer()
					.child(pure_emoji);

				if let Some(_) = emoji.skin_tones {
					let popover_state = cx.global::<PopoverState>();
					let is_open = popover_state.open_emoji == Some(emoji);
					let popover_content =
						variants::types::Variants { font_size, available_emoji: emoji.variants.into() };

					let wrapper = EmojiWrapper {
						content: base_element
							.on_mouse_down(MouseButton::Right, move |_, _, cx: &mut App| {
								cx.update_global::<PopoverState, _>(|state, _| {
									state.open_emoji = Some(emoji);
								});
							})
							.on_click(move |_, _, cx: &mut App| {
								let state = cx.global::<PopoverState>();
								if state.open_emoji == Some(emoji) {
									return;
								}
								close_and_insert(pure_emoji, cx);
							})
							.into_any_element(),
						selected: is_open,
					};

					Popover::new(pure_emoji)
						.trigger(wrapper)
						.open(is_open)
						.on_open_change(move |open, _, cx| {
							if !open {
								cx.update_global::<PopoverState, _>(|state, _| {
									if state.open_emoji == Some(emoji) {
										state.open_emoji = None;
									}
								});
							}
						})
						.child(popover_content.render(window, cx))
						.into_any_element()
				} else {
					base_element
						.on_click(move |_, _, cx: &mut App| {
							close_and_insert(pure_emoji, cx);
						})
						.into_any_element()
				}
			}))
	}
}
