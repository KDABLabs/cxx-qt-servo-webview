// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

mod browser;
mod embedder;
mod events_loop;
mod swapchain;
mod renderer;
mod servothread;
mod webviewfbo;
mod windowheadless;

fn main() {
    std::env::set_var("QSG_RHI_BACKEND", "opengl");
    // std::env::set_var("QSG_RENDER_LOOP", "basic");

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/kdab/servo/qml/main.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
