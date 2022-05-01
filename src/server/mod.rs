//! Webserver functionality

mod auth;
mod error;
mod routes;

use std::{collections::BTreeMap, sync::Arc};

use auth::JwtKeys;
use axum::{http::StatusCode, routing, Extension, Router};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::{rooms::RoomState, settings::Settings};

/// Type of room cache, saving room data.
type RoomDb = Arc<Mutex<BTreeMap<Uuid, Arc<RoomState>>>>;

/// Webserver routes
pub fn routes(settings: Settings) -> Router {
	let room_db = RoomDb::default();
	let jwt_keys = JwtKeys::from_secret(&settings.jwt_secret);

	Router::new()
		// Backend routes
		.route("/login", routing::post(routes::login))
		.route("/:room_id/ws", routing::get(routes::websocket_upgrade))
		// Frontend routes
		.route(
			"/index.js",
			routing::get_service(ServeFile::new("frontend/index.js")).handle_error(handle_error),
		)
		.route(
			"/bulma.css",
			routing::get_service(ServeFile::new("frontend/bulma.css")).handle_error(handle_error),
		)
		.nest(
			"/pkg",
			routing::get_service(ServeDir::new("frontend/pkg")).handle_error(handle_error),
		)
		.fallback(
			routing::get_service(ServeFile::new("frontend/index.html")).handle_error(handle_error),
		)
		// Layers
		.layer(Extension(jwt_keys))
		.layer(Extension(room_db))
}

/// Handle errors when serving files.
#[allow(clippy::unused_async)] // Is axum handler.
async fn handle_error(err: std::io::Error) -> (StatusCode, String) {
	(StatusCode::INTERNAL_SERVER_ERROR, format!("Could not serve file: {}", err))
}
