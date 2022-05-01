//! Server errors

use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

/// Server error type
#[derive(Debug, Error)]
pub enum ServerError {
	/// JWT error
	#[error("JWT error: {0}")]
	Jwt(#[from] jsonwebtoken::errors::Error),
	/// Invalid input error
	#[error("Invalid input given: {0}")]
	InvalidInput(String),
}

impl IntoResponse for ServerError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::Jwt(err) => {
				(StatusCode::INTERNAL_SERVER_ERROR, format!("Could not encode JWT token: {err}"))
					.into_response()
			}
			Self::InvalidInput(err) => {
				(StatusCode::BAD_REQUEST, format!("Invalid input given: {err}")).into_response()
			}
		}
	}
}
