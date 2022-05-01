# Frontend

## Build standalone package

You need `wasm-pack`:

```bash
cargo install wasm-pack
```

Then build with:

```bash
wasm-pack build --release --target web
```

## Deployment and Usage

This is automatically built and deployed/used when compiling and running the backend and the backend server will serve this frontend. It is simply statically hosted.
