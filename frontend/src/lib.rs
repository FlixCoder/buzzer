//! WebAssembly library of this app.

mod components;
mod routes;

use std::panic;

use color_eyre::Result;
use wasm_bindgen::prelude::*;

/// Actual main function
pub fn run() -> Result<()> {
	color_eyre::install()?;

	let _handle = yew::start_app::<routes::AppRouter>();

	Ok(())
}

/// WebAssembly main function
#[wasm_bindgen]
pub fn wasm_main() {
	// This hook is necessary to get panic messages in the console
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	// Set up logging
	wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

	if let Err(err) = run() {
		log::error!("Error running the app: {err}");
	}
}
