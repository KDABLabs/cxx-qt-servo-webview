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
            qml_files: &["qml/main.qml"],
            ..Default::default()
        })
        .cc_builder(|cc| {
            cc.include("cpp");
            cc.file("cpp/helpers.cpp");
        })
        .file("src/renderer.rs")
        .qobject_header("cpp/helpers.h")
        .with_opts(cxx_qt_lib_headers::build_opts())
        .build();
}
