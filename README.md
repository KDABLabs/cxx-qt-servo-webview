

# rustc

We need a version of Rust that matches what Servo builds with.

```console
rustup install nightly-2023-04-01
rustup default nightly-2023-04-01
```

Might need

```console
rustup components add llvm-tools rustc-dev
```

# cargo.lock

We need to copy the cargo.lock from the servo repository to ensure we have the correct patched versions of crates.

# logging

```
RUST_LOG="debug" cargo un
```

# servo

```console
git clone servo
cd servo
git checkout 117d59d393cf7926063e8723934fec97fd61d713
```

# design

```
ServoWebView
  - QQuickItem
  - provides QML API for URLs etc
  - Creates and syncs to a QServoGLRenderNode

QServoGLRenderNode
  - contains the actual servo engine
  - on render initialises webrender with the OpenGL context
```
