//! API types

pub mod websocket;

use serde::{Deserialize, Serialize};

/// Authentication cookie name.
pub const AUTH_COOKIE: &str = "user_token";

/// Login information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
	/// Username
	pub username: String,
}

impl LoginInfo {
	/// Validate the login info.
	#[must_use]
	pub fn is_valid(&self) -> bool {
		!self.username.is_empty()
	}
}

/// Login response with the token for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
	/// Login token
	pub token: String,
}
