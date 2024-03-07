// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qt_module("Network")
        .qt_module("Quick")
        .qt_module("OpenGL")
        .qml_module(QmlModule {
            uri: "com.kdab.servo",
            rust_files: &["src/webview.rs"],
            qml_files: &[
                "qml/main.qml",
                "qml/KdabPanel.qml",
                "qml/InfoPanel.qml",
                "qml/ServoToolbar.qml",
                "qml/WarningPanel.qml",
            ],
            qrc_files: &[
                "images/arrow-back.png",
                "images/arrow-forward.png",
                "images/favicon-kdab.png",
                "images/favicon-qt.png",
                "images/favicon-rust.png",
                "images/favicon-servo.png",
                "images/kdab-logo.png",
                "images/rust-logo-white.png",
                "images/servo-color-negative-no-container.svg",
                "images/search.png",
            ],
            ..Default::default()
        })
        .cc_builder(|cc| {
            cc.include("cpp");
            cc.file("cpp/helpers.cpp");
            println!("cargo:rerun-if-changed=cpp/helpers.cpp");
        })
        .file("src/renderer.rs")
        .qobject_header("cpp/helpers.h")
        .with_opts(cxx_qt_lib_headers::build_opts())
        .build();
}
