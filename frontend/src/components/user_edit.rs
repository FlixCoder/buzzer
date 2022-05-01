//! User editor page component

use api_types::{LoginInfo, LoginResponse};
use reqwasm::http::Request;
use wasm_bindgen::UnwrapThrowExt;
use yew::{html, Callback, Component};
use yew_router::{history::History, prelude::RouterScopeExt};

use super::{get_value_from_input_event, GlobalStateAction, UserData};
use crate::routes::{GlobalContext, Routes};

/// Actions the user can trigger
pub enum Actions {
	/// Start saving user info
	Save,
	/// Save the new information to the global state and go back to the index
	/// page
	SaveGoBack(UserData),
	/// Input value change of name field
	InputNameValue(String),
}

/// User edit component
#[derive(Debug, Default)]
pub struct UserEdit {
	/// Value of the name input field
	input_name: String,
	/// Additional CSS class of the input field
	input_class: Option<&'static str>,
}

impl Component for UserEdit {
	type Message = Actions;
	type Properties = ();

	fn create(_ctx: &yew::Context<Self>) -> Self {
		Self::default()
	}

	#[allow(clippy::expect_used)] // used for global history setup
	fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Actions::Save => {
				let login_info = LoginInfo { username: self.input_name.clone() };
				if login_info.is_valid() {
					ctx.link()
						.callback_future_once(|login_info: LoginInfo| async move {
							// Login with the new name
							let response = Request::post("/login")
								.body(
									serde_json::to_string(&login_info)
										.expect_throw("serialize JSON"),
								)
								.header("Content-Type", "application/json")
								.send()
								.await
								.expect_throw("login request");

							let login_resp: LoginResponse = response
								.json()
								.await
								.expect_throw("deserialize JSON from response");
							Actions::SaveGoBack(UserData {
								name: login_info.username,
								token: login_resp.token,
							})
						})
						.emit(login_info);

					self.input_class = Some("is-loading");
					true
				} else {
					self.input_class = Some("is-danger");
					true
				}
			}
			Actions::SaveGoBack(user_data) => {
				// Set global state to new user data
				ctx.link()
					.context::<GlobalContext>(Callback::noop())
					.expect_throw("context must be provided")
					.0
					.dispatch(GlobalStateAction::ReplaceUser(user_data));

				// Go back to index page
				let history = ctx.link().history().expect("access to history");
				history.push(Routes::Index);

				false
			}
			Actions::InputNameValue(value) => {
				self.input_name = value;
				let login_info = LoginInfo { username: self.input_name.clone() };
				if !login_info.is_valid() {
					self.input_class = Some("is-warning");
				} else {
					self.input_class = Some("is-success");
				}
				true
			}
		}
	}

	fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
		let save = ctx.link().callback(|_e| Actions::Save);
		let input_change =
			ctx.link().callback(|event| Actions::InputNameValue(get_value_from_input_event(event)));

		let classes: String =
			["input"].iter().chain(&self.input_class).flat_map(|s| [*s, " "]).collect();

		html! {
			<>
			<section class="hero">
				<p class="hero-body title">{ "Enter your name" }</p>
			</section>
			<form class="section" onsubmit={save} action="javascript:void(0);">
				<div class="field">
					<div class="control">
						<input class={classes} type="text" placeholder="<name>" oninput={input_change} />
					</div>
				</div>
				<div class="field">
					<div class="control">
						<input type="submit" class="button is-fullwidth" value="Save" />
					</div>
				</div>
			</form>
			</>
		}
	}
}
