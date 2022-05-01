# Buzzer

Simple web-app to have a gameshow-buzzer to play with friends.

It uses web-sockets and UUID-based rooms to open buzzer lobbies. The first person in a room is the host.

## Usage

Copy `config.sample.yaml` to `config.yaml` and adjust the settings.
Then run `cargo run` or `cargo run --release` to launch the app and open it in the browser.

The compilation will require `wasm-pack` to be installed:

```bash
cargo install wasm-pack
```

## Lints

Install [`cargo-lints`](https://github.com/soramitsu/iroha2-cargo_lints) using `cargo install --git https://github.com/FlixCoder/cargo-lints`. The lints are defined in `lints.toml` and can be checked by running `cargo lints clippy --all-targets --workspace`.
