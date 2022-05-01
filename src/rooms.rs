//! Buzzer rooms

use std::collections::HashMap;

use api_types::{
	websocket::{self, ServerMessage},
	LoginInfo,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

/// User data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
	/// Username
	pub name: String,
}

impl From<LoginInfo> for UserData {
	fn from(login: LoginInfo) -> Self {
		Self { name: login.username }
	}
}

/// Buzzer room state. Lock in the order that is written here to avoid
/// deadlocks.
#[derive(Debug)]
pub struct RoomState {
	/// Members currently active in the room.
	members: RwLock<HashMap<String, UserData>>,
	/// Current host of the room/session,
	host: RwLock<String>,
	/// The user who buzzed
	buzzed: RwLock<Option<String>>,
	/// Event sender (publisher)
	event_pub: broadcast::Sender<ServerMessage>,
}

impl Default for RoomState {
	fn default() -> Self {
		let (sender, _receiver) = broadcast::channel(5);
		Self {
			members: RwLock::default(),
			host: RwLock::default(),
			buzzed: RwLock::default(),
			event_pub: sender,
		}
	}
}

impl RoomState {
	/// Subscribe to the event stream of this room.
	#[inline]
	pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
		self.event_pub.subscribe()
	}

	/// Get the members
	#[inline]
	pub async fn members(&self) -> HashMap<String, UserData> {
		self.members.read().await.clone()
	}

	/// Get the number of members currently in the room.
	#[inline]
	pub async fn num_members(&self) -> usize {
		self.members.read().await.len()
	}

	/// Get whether the room is currently empty
	#[inline]
	pub async fn is_empty(&self) -> bool {
		self.num_members().await == 0
	}

	/// Get the current host of the room.
	#[inline]
	pub async fn host(&self) -> String {
		self.host.read().await.clone()
	}

	/// Set the host to the specified person.
	#[inline]
	async fn set_host(&self, name: &str) -> &Self {
		*self.host.write().await = name.to_owned();
		self
	}

	/// Get the current buzzing person of the room.
	#[inline]
	pub async fn buzzed(&self) -> Option<String> {
		self.buzzed.read().await.clone()
	}

	/// Set the buzzing person to the specified.
	#[inline]
	pub async fn set_buzzed(&self, name: impl Into<Option<String>>) -> Option<&Self> {
		let buzzed = name.into();
		*self.buzzed.write().await = buzzed.clone();
		self.event_pub.send(ServerMessage::Buzzed(buzzed)).ok();
		Some(self)
	}

	/// Get state of the room
	pub async fn state(&self) -> websocket::RoomState {
		websocket::RoomState {
			members: self.members().await.into_keys().collect(),
			host: self.host().await,
			buzzed: self.buzzed().await,
		}
	}

	/// Join a new member if it doesn't exist and return the current amount of
	/// members after that operation if successful. None means somebody with
	/// that name was already present.
	pub async fn join_member(&self, user: UserData) -> Option<usize> {
		let mut members = self.members.write().await;
		if members.get(&user.name).is_some() {
			return None;
		}

		if members.len() == 0 {
			self.set_host(&user.name).await;
		}
		members.insert(user.name.clone(), user);
		let num_members = members.len();
		drop(members);

		self.event_pub.send(ServerMessage::State(self.state().await)).ok();

		Some(num_members)
	}

	/// Leave a member if it is there and return the current amount of members
	/// after that operation if successful. None means it was not there to
	/// leave.
	pub async fn leave_member(&self, name: &String) -> Option<usize> {
		let mut members = self.members.write().await;
		let prev = members.remove(name);

		if let Some(prev) = prev.as_ref() {
			if prev.name == self.host().await {
				let first = members.iter().next().map_or("", |(_, user)| &user.name);
				self.set_host(first).await;
			}
		}

		let out = prev.map(|_old| members.len());
		drop(members);

		self.event_pub.send(ServerMessage::State(self.state().await)).ok();

		out
	}
}
