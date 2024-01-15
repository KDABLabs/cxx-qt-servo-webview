# Servo WebView for Qt using CXX-Qt

TODO

# Setup

We need the nightly compiler and a specific version of Servo for now.

```console
rustup install nightly
rustup default nightly

git clone servo
cd servo
git checkout 117d59d393cf7926063e8723934fec97fd61d713
```

# Notes

## cargo.lock

We need to copy the cargo.lock from the servo repository to ensure we have the correct patched versions of crates.

## Logging

```console
RUST_LOG="debug" cargo run
```

## Rust

We might need

```console
rustup components add llvm-tools rustc-dev
```
