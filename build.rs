//! Build file
#![allow(clippy::expect_used)]

use std::process::Command;

fn main() {
	let frontend_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend");

	println!("cargo:rerun-if-changed={frontend_dir}/src");
	println!("cargo:rerun-if-changed={frontend_dir}/Cargo.toml");

	let build = Command::new("wasm-pack")
		.args(["build", "--release", "--target", "web"])
		.current_dir(frontend_dir)
		.status()
		.expect("running frontend build");
	assert!(build.success());
}
