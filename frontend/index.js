import init, { wasm_main } from "./pkg/frontend.js";

async function run() {
	await init();
	wasm_main();
}

run();
