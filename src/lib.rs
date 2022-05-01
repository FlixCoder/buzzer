//! Crate library

mod rooms;
mod server;
mod settings;

use axum::Router;
use color_eyre::Result;
use settings::Settings;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::EnvFilter;

/// Main function
pub async fn run() -> Result<()> {
	color_eyre::install()?;
	let settings = Settings::read()?;

	let filter = EnvFilter::from_default_env()
		.add_directive(settings.log_level.into())
		.add_directive("hyper=info".parse()?)
		.add_directive("mio=info".parse()?)
		.add_directive("want=info".parse()?)
		.add_directive("tungstenite=info".parse()?)
		.add_directive("tokio=info".parse()?)
		.add_directive("sqlx=error".parse()?);
	tracing_subscriber::fmt().with_env_filter(filter).init();

	// Start server
	tracing::info!("Starting server and listening on {}", settings.bind);
	axum::Server::bind(&settings.bind).serve(server_app(settings).into_make_service()).await?;

	Ok(())
}

/// Webserver routes
#[must_use]
pub fn server_app(settings: Settings) -> Router {
	server::routes(settings).layer(
		TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
	)
}
