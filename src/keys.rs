//! Action definitions and keybind registration for the picker.

use gpui::{actions, Action, App, KeyBinding};
use serde::Deserialize;

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = input, no_json)]
pub struct JumpToSection {
	pub number: usize,
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = input, no_json)]
pub struct RotateTones {
	pub direction: Direction,
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub enum Direction {
	Forward,
	Backward,
}

actions!(theme, [SwitchToLight, SwitchToDark]);
actions!(all, [Quit, Cancel]);
actions!(picker, [
	MoveUp,
	MoveDown,
	MoveLeft,
	MoveRight,
	SelectCurrent,
	OpenSecondary,
	FocusSearch,
]);

pub(crate) fn bind_all(cx: &mut App) {
	use Direction::{Backward, Forward};

	let mut bindings = vec![
		KeyBinding::new("super-q", Quit, None),
		KeyBinding::new("super-w", Quit, None),
		KeyBinding::new("escape", Cancel, None),
		KeyBinding::new("enter", Cancel, None),

		KeyBinding::new("up", MoveUp, Some("List")),
		KeyBinding::new("down", MoveDown, Some("List")),
		KeyBinding::new("left", MoveLeft, Some("List")),
		KeyBinding::new("right", MoveRight, Some("List")),
		KeyBinding::new(",", OpenSecondary, Some("List")),
		KeyBinding::new("shift-space", SelectCurrent, Some("List")),

		KeyBinding::new("space", SelectCurrent, Some("ListBody")),
		KeyBinding::new("N", RotateTones { direction: Backward }, Some("ListBody")),
		KeyBinding::new("n", RotateTones { direction: Forward }, Some("ListBody")),
		KeyBinding::new("k", MoveUp, Some("ListBody")),
		KeyBinding::new("j", MoveDown, Some("ListBody")),
		KeyBinding::new("h", MoveLeft, Some("ListBody")),
		KeyBinding::new("l", MoveRight, Some("ListBody")),
		KeyBinding::new("/", FocusSearch, Some("ListBody")),
	];

	for n in 0..=9 {
		bindings.push(KeyBinding::new(
			&format!("super-{n}"),
			JumpToSection { number: n },
			None,
		));
	}

	cx.bind_keys(bindings);
}
