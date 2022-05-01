//! Frontend view components

pub mod index;
pub mod room;
pub mod user_edit;

use std::rc::Rc;

pub use index::Index;
pub use room::Room;
pub use user_edit::UserEdit;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::{InputEvent, Reducible};

/// Global context data/state
#[derive(Debug, PartialEq, Eq, Default)]
pub struct GlobalState {
	/// User data
	pub user: Option<UserData>,
}

/// User data
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserData {
	/// Username
	pub name: String,
	/// Login token
	pub token: String,
}

/// Actions onto the global state.
pub enum GlobalStateAction {
	/// Set the `user` field to the `UserData`.
	ReplaceUser(UserData),
}

impl Reducible for GlobalState {
	type Action = GlobalStateAction;

	#[allow(clippy::infallible_destructuring_match)] // can be extended
	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		let next = match action {
			GlobalStateAction::ReplaceUser(user_data) => user_data,
		};
		Self { user: Some(next) }.into()
	}
}

/// Extract the `<input />`'s value from the [`InputEvent`].
fn get_value_from_input_event(event: InputEvent) -> String {
	let event_target = event.target().unwrap_throw();
	let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
	target.value()
}
