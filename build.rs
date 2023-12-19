use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    println!("cargo:rerun-if-changed={}", "include/platformhelpers.h");
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
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtGui/6.5.0");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtGui/6.5.0/QtGui");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtCore/6.5.0");
            cc.include("/var/home/andrew/.var/Qt/6.5.0/gcc_64/include/QtCore/6.5.0/QtCore");

            cc.file("cpp/qservoglrendernode.cpp");
            cc.file("cpp/qservorendernode.cpp");
        })
        .qobject_header("cpp/qservoglrendernode.h")
        .qobject_header("cpp/qservorendernode.h")
        .file("src/window.rs")
        .build();
}
