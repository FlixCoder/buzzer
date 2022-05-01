//! Executable entry

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
	buzzer::run().await
}
