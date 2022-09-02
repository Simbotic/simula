## Prerequisites

```
rustup target install wasm32-unknown-unknown
```

```
cargo install wasm-server-runner
```

## Build and serve

```
cargo run --target wasm32-unknown-unknown
```

then point your browser to http://127.0.0.1:1334/

Note: you also need to run the simula_server at http://127.0.0.1:3536