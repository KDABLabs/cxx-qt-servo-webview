<!--
SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>

SPDX-License-Identifier: MPL-2.0
-->

# Servo WebView for Qt using CXX-Qt

TODO

# Setup

We need the nightly compiler for now.

```console
rustup install nightly-2024-02-01
rustup default nightly-2024-02-01
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
