use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    println!("cargo:rerun-if-changed={}", "include/platformhelpers.h");
    CxxQtBuilder::new()
        .qt_module("Network")
        .qml_module(QmlModule {
            uri: "com.kdab.servo",
            rust_files: &["src/webview.rs"],
            qml_files: &["qml/main.qml"],
            ..Default::default()
        })
        .cc_builder(|cc| {
            cc.include("include");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtGui/6.5.0");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtGui/6.5.0/QtGui");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtCore/6.5.0");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtCore/6.5.0/QtCore");
        })
        .file("src/window.rs")
        .build();
}
