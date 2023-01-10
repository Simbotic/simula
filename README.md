# Simula - Simbotic

## 3D engine for vectorizing meta-layers for XR experiences
So much of AI is about compressing reality to a small vector space, like a video game in reverse.

![Simula splash](docs/splash.png)

![Simula splash gif](docs/Simula__main.gif)

### Behaviors AI
![Simula splash gif](docs/behaviors.gif)

### Easings Function

![Easings Function](docs/easings_functions.png)

### Signal Generators

![Signal Generators](docs/signal_generators.png)

### Control Signals

![Control Signals](docs/control_signals.png)

### Force Graph Function

![Force Graph Function](docs/force_graph_function.png)

### P2P Networking

![P2P Networking](docs/p2p.png)

## WASM
To add WASM support to your Rust installation. Using Rustup:
```
rustup target install wasm32-unknown-unknown
```

Now, to run your project in the browser.
```
wasm-server-runner
```

The easiest and most automatic way to get started is the wasm-server-runner tool.

Install it:
```
cargo install wasm-server-runner
```

Set up cargo to use it, in `.cargo/config.toml` (in your project folder, or globally in your user home folder):
```
[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```

Alternatively, you can also set the runner using an environment variable:

```
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner
```

Now you can just run your simulation with:
```
cargo run --target wasm32-unknown-unknown
```

It will automatically run a minimal local webserver and open your simulation in your browser.

## P2P Networking

Launch signaling server:
```
cargo run --release -p simula_signaling
```

Launch any number of native peers:
```
cargo run --release -p net_peer
```

Launch any number of WASM peers:
```
cargo run --target wasm32-unknown-unknown --release -p net_peer
```

## GStreamer

### Ubuntu
```
cargo run --release -p sandbox --features gif,gst
```

### OSX
To get started with GStreamer in macOS, both the runtime and development packages, where they must be installed via the official GStreamer website: https://gstreamer.freedesktop.org/download/#macos

In addition to install GStreamer, also it’s necessary install GStreamer plugins via Homebrew:
```
brew install gst-plugins-base
```
```
brew install gst-plugins-rs
```
After installation, you also need to install `pkg-config` (e.g. via Homebrew) and set the `PKG_CONFIG_PATH` environment variable
```
export PKG_CONFIG_PATH="/Library/Frameworks/GStreamer.framework/Versions/1.0/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
```
And finally run this command in the projects terminal
```
DYLD_LIBRARY_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib cargo run --release -p sandbox --features gif,gst
```
