//! Webserver handlers
#![allow(clippy::unused_async)]

use api_types::{
	websocket::{ClientMessage, ServerMessage},
	LoginInfo, LoginResponse,
};
use axum::{
	extract::{
		ws::{Message, WebSocket},
		Path, WebSocketUpgrade,
	},
	response::IntoResponse,
	Extension, Json,
};
use axum_macros::debug_handler;
use uuid::Uuid;

use super::{
	auth::{Authentication, Claims, JwtKeys},
	error::ServerError,
	RoomDb,
};
use crate::rooms::{RoomState, UserData};

/// Handler for "login", i.e. setting user data and receiving a token for
/// authentication.
#[debug_handler]
pub async fn login(
	jwt_keys: Extension<JwtKeys>,
	Json(login_info): Json<LoginInfo>,
) -> Result<Json<LoginResponse>, ServerError> {
	if !login_info.is_valid() {
		return Err(ServerError::InvalidInput("Invalid user data!".to_owned()));
	}

	let user_data = UserData::from(login_info);
	let claims = Claims::new(user_data);
	let token = jwt_keys.encode_jwt(&claims)?;
	Ok(Json(LoginResponse { token }))
}

/// Handler for upgrading to web-sockets
#[debug_handler]
pub async fn websocket_upgrade(
	room_db: Extension<RoomDb>,
	claims: Authentication,
	ws: WebSocketUpgrade,
	Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
	ws.on_upgrade(move |ws| async move {
		let room = room_db.lock().await.entry(room_id).or_default().clone();
		let user_data = claims.0.user_data;

		tracing::debug!("Connecting websocket user.");
		websocket_handler(ws, &room, &user_data).await;
		tracing::debug!("Disconnecting websocket user.");

		room.leave_member(&user_data.name).await;
		if room.is_empty().await {
			room_db.lock().await.remove(&room_id);
		}
	})
}

/// Websocket handlers
async fn websocket_handler(
	mut ws: WebSocket,
	room: &RoomState,
	user_data: &UserData,
) -> Option<()> {
	room.join_member(user_data.clone()).await?;
	let state = ServerMessage::State(room.state().await);
	ws.send(Message::Text(serde_json::to_string(&state).ok()?)).await.ok()?;

	let mut events = room.subscribe();
	loop {
		tokio::select! {
			event = events.recv() => {
				tracing::trace!("Sending room event via websocket..");
				ws.send(Message::Text(serde_json::to_string(&event.ok()?).ok()?)).await.ok()?;
			}

			msg = ws.recv() => match msg? {
				Ok(Message::Text(msg)) => {
					tracing::trace!("Received message via websocket..");
					let client_msg: ClientMessage = serde_json::from_str(&msg).ok()?;
					handle_client_message(room, user_data, client_msg).await?;
				}

				Ok(Message::Close(_)) | Err(_) => break,
				_ => {}
			}
		}
	}

	None
}

/// Handle a message from a client
async fn handle_client_message(
	room: &RoomState,
	user_data: &UserData,
	client_msg: ClientMessage,
) -> Option<()> {
	let is_host = room.host().await == user_data.name;
	match client_msg {
		ClientMessage::Buzz => {
			if room.buzzed().await.is_none() {
				room.set_buzzed(Some(user_data.name.clone())).await?;
			}
		}
		ClientMessage::FreeBuzzer => {
			if is_host {
				room.set_buzzed(None).await?;
			}
		}
		ClientMessage::Leave => {
			return None;
		}
	};
	Some(())
}
