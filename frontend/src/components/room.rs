//! Room page component

use api_types::{
	websocket::{ClientMessage, RoomState, ServerMessage},
	AUTH_COOKIE,
};
use futures::{SinkExt, StreamExt};
use reqwasm::websocket::{futures::WebSocket, Message};
use tokio::sync::mpsc;
use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;
use wasm_cookies::CookieOptions;
use yew::{html, Callback, Component, Html, Properties};
use yew_router::{history::History, prelude::RouterScopeExt};

use super::UserData;
use crate::routes::{GlobalContext, Routes};

/// Actions the user can trigger
pub enum Actions {
	/// No action
	None,
	/// Leave the room and go back to index
	LeaveRoom,
	/// Press the buzzer
	Buzz,
	/// Free the buzzer
	FreeBuzzer,
	/// Reconnect the websocket
	Reconnect,
	/// Someone buzzed
	Buzzed(Option<String>),
	/// New room state
	RoomState(RoomState),
}

/// Properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Properties)]
pub struct Props {
	/// Room ID
	pub id: Uuid,
}

/// Room component
pub struct Room {
	/// Websocket connection sender
	ws: mpsc::UnboundedSender<Message>,
	/// State of the room
	state: RoomState,
	/// This person's name
	my_name: String,
	/// This user's login token
	my_token: String,
}

impl Room {
	/// Connect to the room's websocket
	pub fn connect_ws(
		room_id: Uuid,
		token: &str,
		on_message: Callback<Message>,
	) -> mpsc::UnboundedSender<Message> {
		let location = web_sys::window().expect_throw("access to window").location();
		let proto = if location.protocol().ok().as_deref() == Some("https") { "wss" } else { "ws" };
		let host = location.host().expect_throw("access to window.location.host");
		let path = format!("{proto}://{host}/{}/ws", room_id);

		let options = CookieOptions::default().with_path("/");
		wasm_cookies::set(AUTH_COOKIE, token, &options);

		let ws = WebSocket::open(&path).expect_throw("connecting to room websocket");
		let (mut sender, mut receiver) = ws.split();
		let (channel_sender, mut channel_receiver) = mpsc::unbounded_channel();

		spawn_local(async move {
			while let Some(msg) = receiver.next().await {
				on_message.emit(msg.expect_throw("receiving websocket message"));
			}
		});

		spawn_local(async move {
			while let Some(msg) = channel_receiver.recv().await {
				sender.send(msg).await.expect_throw("sending websocket message");
			}
		});

		channel_sender
	}

	/// What to do on a websocket message
	fn on_message(msg: Message) -> Actions {
		match msg {
			Message::Text(msg) => {
				let event: ServerMessage =
					serde_json::from_str(&msg).expect_throw("deserializing message");
				match event {
					ServerMessage::State(state) => Actions::RoomState(state),
					ServerMessage::Buzzed(buzzed) => Actions::Buzzed(buzzed),
				}
			}
			_ => Actions::None,
		}
	}
}

impl Component for Room {
	type Message = Actions;
	type Properties = Props;

	#[allow(clippy::expect_used)] // used for global history setup
	fn create(ctx: &yew::Context<Self>) -> Self {
		let global_state = ctx
			.link()
			.context::<GlobalContext>(Callback::noop())
			.expect_throw("getting global state")
			.0;
		let user = if let Some(user) = global_state.user.as_ref() {
			user.clone()
		} else {
			ctx.link().history().expect("accessing history").push(Routes::UserEdit);
			UserData::default()
		};

		let on_message = ctx.link().callback(Room::on_message);
		let ws = Room::connect_ws(ctx.props().id, &user.token, on_message);
		Self { ws, state: RoomState::default(), my_name: user.name, my_token: user.token }
	}

	#[allow(clippy::expect_used)] // used for global history setup
	fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Actions::None => false,
			Actions::LeaveRoom => {
				let msg = serde_json::to_string(&ClientMessage::Leave)
					.expect_throw("serializing message");
				self.ws.send(Message::Text(msg)).expect_throw("sending message");
				let history = ctx.link().history().expect("access to history");
				history.push(Routes::Index);
				false
			}
			Actions::Buzz => {
				let msg =
					serde_json::to_string(&ClientMessage::Buzz).expect_throw("serializing message");
				self.ws.send(Message::Text(msg)).expect_throw("sending message");
				false
			}
			Actions::FreeBuzzer => {
				let msg = serde_json::to_string(&ClientMessage::FreeBuzzer)
					.expect_throw("serializing message");
				self.ws.send(Message::Text(msg)).expect_throw("sending message");
				false
			}
			Actions::Reconnect => {
				let on_message = ctx.link().callback(Room::on_message);
				self.ws = Room::connect_ws(ctx.props().id, &self.my_token, on_message);
				false
			}
			Actions::RoomState(state) => {
				self.state = state;
				true
			}
			Actions::Buzzed(buzzed) => {
				self.state.buzzed = buzzed;
				true
			}
		}
	}

	fn view(&self, ctx: &yew::Context<Self>) -> Html {
		// TODO: Add keyboard listener by batch_callback.

		let leave = ctx.link().callback(|_e| Actions::LeaveRoom);
		let reconnect = ctx.link().callback(|_e| Actions::Reconnect);
		let buzz = ctx.link().callback(|_e| Actions::Buzz);
		let free_buzzer = ctx.link().callback(|_e| Actions::FreeBuzzer);

		let main_stage = if let Some(buzzed) = self.state.buzzed.as_ref() {
			html! {
				<>
				{ buzzed }{ " has buzzed!" }
				if self.my_name == self.state.host {
					<button class="button" onclick={free_buzzer}>{ "Free Buzzer" }</button>
				}
				</>
			}
		} else {
			html! {
				<button class="button is-danger is-rounded is-large" onclick={buzz}>
					{ "Buzz!" }
				</button>
			}
		};

		let members = self
			.state
			.members
			.iter()
			.map(|name| {
				html! {
					<tr><td>
						if *name == self.my_name {
							<strong>{ name }</strong>
						} else {
							{ name }
						}
					</td></tr>
				}
			})
			.collect::<Html>();

		html! {
			<>
			<div class="columns is-fullheight">
				<div class="column is-four-fifths">
					<div class="section is-fullheight is-large">
						<div class="container is-fullheight is-vcentered is-centered">
							{ main_stage }
						</div>
					</div>
				</div>
				<div class="column content is-medium">
					<button class="button" onclick={leave}>{ "Leave" }</button>
					<button class="button" onclick={reconnect}>{ "Reconnect" }</button>
					<table>
						<thead><tr><td>
							{ "Room Members" }
						</td></tr></thead>
						{ members }
					</table>
				</div>
			</div>
			</>
		}
	}
}
