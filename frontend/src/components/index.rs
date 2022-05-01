//! Index page component

use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;
use yew::{context::ContextHandle, html, Component};
use yew_router::{history::History, prelude::RouterScopeExt};

use super::get_value_from_input_event;
use crate::routes::{GlobalContext, Routes};

/// Actions the user can trigger
pub enum Actions {
	/// Create a new room (join random room)
	CreateRoom,
	/// Join a specific, given room
	JoinRoom,
	/// Go to user edit page
	EditUser,
	/// Input value change
	InputValue(String),
	/// Context was updated
	ContextChange(GlobalContext),
}

/// Index component
#[derive(Debug)]
pub struct Index {
	/// Room ID to join
	room_id: Option<Uuid>,
	/// Additional CSS class of the input field
	input_class: Option<&'static str>,
	/// Global state
	global_state: GlobalContext,
	/// Global state context listener
	_context_listener: ContextHandle<GlobalContext>,
}

impl Component for Index {
	type Message = Actions;
	type Properties = ();

	#[allow(clippy::expect_used)] // used for global history setup
	fn create(ctx: &yew::Context<Self>) -> Self {
		let history = ctx.link().history().expect("access to history");

		let (global_state, listener) = ctx
			.link()
			.context(ctx.link().callback(Actions::ContextChange))
			.expect_throw("Context must be there");

		if global_state.user.is_none() {
			history.push(Routes::UserEdit);
		}

		Self { room_id: None, input_class: None, global_state, _context_listener: listener }
	}

	#[allow(clippy::expect_used)] // used for global history setup
	fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Actions::CreateRoom => {
				let id = Uuid::new_v4();
				self.room_id = Some(id);
				let history = ctx.link().history().expect("access to history");
				history.push(Routes::Room { id });
				false
			}
			Actions::JoinRoom => {
				if let Some(&id) = self.room_id.as_ref() {
					let history = ctx.link().history().expect("access to history");
					history.push(Routes::Room { id });
					false
				} else {
					self.input_class = Some("is-danger");
					true
				}
			}
			Actions::EditUser => {
				let history = ctx.link().history().expect("access to history");
				history.push(Routes::UserEdit);
				false
			}
			Actions::InputValue(value) => {
				self.room_id = value.parse().ok();
				if self.room_id.is_none() {
					self.input_class = Some("is-warning");
				} else {
					self.input_class = Some("is-success");
				}
				true
			}
			Actions::ContextChange(new_state) => {
				self.global_state = new_state;
				false
			}
		}
	}

	fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
		let create_room = ctx.link().callback(|_e| Actions::CreateRoom);
		let join_room = ctx.link().callback(|_e| Actions::JoinRoom);
		let input_change =
			ctx.link().callback(|event| Actions::InputValue(get_value_from_input_event(event)));
		let edit_user = ctx.link().callback(|_e| Actions::EditUser);

		let classes: String =
			["input"].iter().chain(&self.input_class).flat_map(|s| [*s, " "]).collect();

		let user_edit_link = (*self.global_state)
			.user
			.as_ref()
			.map_or_else(|| "Logged out (Edit)".to_owned(), |user| format!("{} (Edit)", user.name));

		html! {
			<>
			<button class="button is-pulled-right" onclick={edit_user}>
				{ user_edit_link }
			</button>
			<div class="section">
				<div class="field">
					<div class="control">
						<button class="button" onclick={create_room}>{ "Create Room" }</button>
					</div>
				</div>
				<div class="field has-addons">
					<div class="control">
						<input class={classes} type="text" placeholder="<uuid>" oninput={input_change} />
					</div>
					<div class="control">
						<button class="button" onclick={join_room}>{ "Join Room" }</button>
					</div>
				</div>
			</div>
			</>
		}
	}
}
