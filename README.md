<!--
SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>

SPDX-License-Identifier: MPL-2.0
-->

# Servo WebView for Qt using CXX-Qt

KDAB has built a demo of using [CXX-Qt](https://github.com/KDAB/cxx-qt/) to expose a [Servo](https://servo.org/) in Rust as a component to Qt.

## Setup

* Ensure that you have Qt installed and `qmake` in your `PATH`.
* Ensure that you have the dependencies of `./mach boostrap` from the [https://github.com/servo/servo/](https://github.com/servo/servo/) repository.
* Alternatively use the `shell.nix` by [installing Nix](https://nixos.org/download/) and then running `nix-shell`, it will take care of all dependencies automatically
* Install the nightly compiler from February

```console
$ rustup install nightly-2024-02-01
$ rustup default nightly-2024-02-01
```

* Then run in release mode

```console
$ cargo run --release
```

## Debugging

To make sure, that Qt picks the correct OpenGL driver, use the `QSG_INFO=1` variable. For hardware acceleration to work, the driver name should **not** contain `llvmpipe`.
Note that on embedded hardware it might be necessary to force servo to use OpenGL ES.

### Nix

If you using Nix and if not on NixOS, make sure to run the final executable with the `nixGLMesa` wrapper (or if not on Mesa, with the correct wrapper for your driver). This will make sure that the OpenGL drivers are passed through from the host system with [nixGL](https://github.com/nix-community/nixGL).

## Licensing

This demo is Copyright (C) Klarälvdalens Datakonsult AB, and is available under
the terms of the [MPL-2.0](https://github.com/KDABLabs/cxx-qt-servo-webview/blob/main/LICENSES/MPL-2.0.txt) license.

Contact KDAB at <info@kdab.com> to inquire about additional features or
services related to this project.

# About KDAB

The KDAB Group is the global No.1 software consultancy for Qt, C++ and
OpenGL applications across desktop, embedded and mobile platforms.

The KDAB Group provides consulting and mentoring for developing Qt applications
from scratch and in porting from all popular and legacy frameworks to Qt.
We continue to help develop parts of Qt and are one of the major contributors
to the Qt Project. We can give advanced or standard trainings anywhere
around the globe on Qt as well as C++, OpenGL, 3D and more.

Please visit <https://www.kdab.com> to meet the people who write code like this.
