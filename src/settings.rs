//! Configuration module

use std::net::SocketAddr;

use config::{ConfigError, Environment, File};
use serde::{de::Error, Deserialize, Deserializer};
use tracing::{metadata::ParseLevelError, Level};

/// This app's configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
	/// Webserver bind address and port
	pub bind: SocketAddr,
	/// Logging level
	#[serde(deserialize_with = "deserialize_level")]
	pub log_level: Level,
	/// Secret for initializing JWT keys.
	pub jwt_secret: String,
}

/// Deserialize a Level
fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
	D: Deserializer<'de>,
{
	let log_level = String::deserialize(deserializer)?
		.parse()
		.map_err(|err: ParseLevelError| D::Error::custom(err.to_string()))?;
	Ok(log_level)
}

impl Settings {
	/// Read configuration from `config.yaml` by default. Calls `read_from`.
	#[inline]
	pub fn read() -> Result<Self, ConfigError> {
		Self::read_from("config.yaml")
	}

	/// Read configuration from specified file and merge in environment variable
	/// configuration.
	pub fn read_from(cfg_path: &str) -> Result<Self, ConfigError> {
		config::Config::builder()
			//.set_default("key", "value")?
			.add_source(File::with_name(cfg_path))
			.add_source(Environment::with_prefix("app").separator("__"))
			.build()?
			.try_deserialize()
	}
}
