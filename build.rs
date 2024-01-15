use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qt_module("Network")
        .qt_module("Quick")
        .qt_module("OpenGL")
        .qml_module(QmlModule {
            uri: "com.kdab.servo",
            rust_files: &["src/webviewfbo.rs"],
            qml_files: &["qml/main.qml"],
            ..Default::default()
        })
        .cc_builder(|cc| {
            cc.include("cpp");
            cc.file("cpp/helpers.cpp");
        })
        .file("src/renderer.rs")
        .qobject_header("cpp/helpers.h")
        .build();
}
