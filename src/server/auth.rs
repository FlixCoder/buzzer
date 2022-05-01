//! Authentication and user data store.

use std::{fmt::Debug, ops::Deref, time::Duration};

use api_types::AUTH_COOKIE;
use axum::{
	async_trait,
	extract::{FromRequest, RequestParts},
	headers::{authorization::Bearer, Authorization, Cookie, HeaderMapExt},
	http::{header::WWW_AUTHENTICATE, HeaderMap, StatusCode},
	Extension,
};
use jsonwebtoken::{
	errors::Error, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::rooms::UserData;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
	/// Issued at timestamp
	#[serde(rename = "iat")]
	pub issued_at: i64,
	/// Expiry timestamp
	#[serde(rename = "exp")]
	pub expiry: i64,
	/// User data
	pub user_data: UserData,
}

impl Claims {
	/// JWT encoding algorithm to use
	const ALGORITHM: Algorithm = Algorithm::HS512;
	/// Seconds until tokens expire
	const VALIDITY_SECONDS: u64 = 30 * 24 * 60 * 60; // 1 month

	/// Create new claims
	pub fn new(user_data: UserData) -> Self {
		let now = OffsetDateTime::now_utc();
		let issued_at = now.unix_timestamp();
		let expiry = (now + Duration::from_secs(Self::VALIDITY_SECONDS)).unix_timestamp();
		Self { issued_at, expiry, user_data }
	}

	/// Encode the claims to a JWT token
	pub fn to_jwt(&self, keys: &JwtKeys) -> Result<String, Error> {
		let header = Header::new(Self::ALGORITHM);
		jsonwebtoken::encode(&header, self, &keys.encoding)
	}

	/// Decode the claims from a JWT token
	pub fn from_jwt(jwt: &str, keys: &JwtKeys) -> Result<Self, Error> {
		let validation = Validation::new(Self::ALGORITHM);
		let token_data: TokenData<Self> = jsonwebtoken::decode(jwt, &keys.decoding, &validation)?;
		Ok(token_data.claims)
	}
}

/// Encoding and decoding keys
#[derive(Clone)]
pub struct JwtKeys {
	/// Encoding key
	encoding: EncodingKey,
	/// Decoding key
	decoding: DecodingKey,
}

impl Debug for JwtKeys {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("JwtKeys")
			.field("encoding", &"<redacted>")
			.field("decoding", &"<redacted>")
			.finish()
	}
}

impl JwtKeys {
	/// Generate keys from a secret
	pub fn from_secret(secret: &str) -> Self {
		Self {
			encoding: EncodingKey::from_secret(secret.as_bytes()),
			decoding: DecodingKey::from_secret(secret.as_bytes()),
		}
	}

	/// Encode JWT using these keys
	pub fn encode_jwt(&self, claims: &Claims) -> Result<String, Error> {
		claims.to_jwt(self)
	}

	/// Decode JWT using these keys
	pub fn decode_jwt(&self, jwt: &str) -> Result<Claims, Error> {
		Claims::from_jwt(jwt, self)
	}
}

/// Authentication extractor using Authorization header and a Cookie as
/// fallback.
#[derive(Debug, Clone)]
pub struct Authentication(pub Claims);

#[async_trait]
impl<B: Send> FromRequest<B> for Authentication {
	type Rejection = (StatusCode, HeaderMap, String);

	async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
		let mut headers = HeaderMap::new();
		#[allow(clippy::unwrap_used)] // used on const
		headers.insert(WWW_AUTHENTICATE, "Bearer".parse().unwrap());

		let authorization = req.headers().typed_get::<Authorization<Bearer>>();
		let cookies = req.headers().typed_get::<Cookie>();
		let token = authorization
			.as_ref()
			.map(Authorization::token)
			.or_else(|| cookies.as_ref().and_then(|cookie| cookie.get(AUTH_COOKIE)))
			.ok_or_else(|| {
				(
					StatusCode::UNAUTHORIZED,
					headers.clone(),
					"No authorization header or auth cookie found!".to_owned(),
				)
			})?;

		#[allow(clippy::expect_used)] // Fast failure, can't run at all
		let keys = Extension::<JwtKeys>::from_request(req).await.expect("JWT keys extension must be set!");
		let claims = keys.decode_jwt(token).map_err(|err| {
			(StatusCode::UNAUTHORIZED, headers, format!("Error decoding JWT: {err}"))
		})?;

		Ok(Self(claims))
	}
}

impl Deref for Authentication {
	type Target = Claims;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
