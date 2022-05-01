//! Websocket communication types

use serde::{Deserialize, Serialize};

/// Message type for websocket communication from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
	/// Current state of the room
	State(RoomState),
	/// Who buzzed
	Buzzed(Option<String>),
}

/// Message type for websocket communication from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
	/// Press the buzzer
	Buzz,
	/// Free the buzzer
	FreeBuzzer,
	/// Leave
	Leave,
}

/// Room state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoomState {
	/// Room members (names)
	pub members: Vec<String>,
	/// Host
	pub host: String,
	/// Buzzing person
	pub buzzed: Option<String>,
}
