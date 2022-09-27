# Simula - Simbotic

## 3D engine for vectorizing meta-layers for XR experiences
So much of AI is about compressing reality to a small vector space, like a video game in reverse.

![Simula splash](docs/splash.png)






## GStreamer

### Ubuntu
```
cargo run --release -p sandbox --features gif,gst
```

### OSX
```
DYLD_LIBRARY_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib cargo run --release -p sandbox --features gif,gst
```