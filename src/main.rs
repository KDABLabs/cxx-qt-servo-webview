// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use std::{env, path::Path, thread, time::Duration};

mod browser;
mod embedder;
mod events_loop;
mod renderer;
mod servothread;
mod webview;
mod windowheadless;

fn main() {
    // Start serving files in the background
    thread::spawn(|| {
        let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("static_files");
        file_serve::ServerBuilder::new(path)
            .hostname("0.0.0.0")
            .port(8001)
            .serve()
            .unwrap();
    });

    // Ensure that file serve is ready
    thread::sleep(Duration::from_secs(1));

    // We need the OpenGL backend for QQuickFramebufferObject
    env::set_var("QSG_RHI_BACKEND", "opengl");

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/kdab/servo/qml/main.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
