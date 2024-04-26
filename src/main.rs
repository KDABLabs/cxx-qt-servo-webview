// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

mod browser;
mod embedder;
mod events_loop;
mod renderer;
mod servothread;
mod webview;
mod windowheadless;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("helpers.h");

        #[cxx_name = "forceSurfaceFormat"]
        fn force_surface_format();
    }
}

fn main() {
    // We need the OpenGL backend for QQuickFramebufferObject
    std::env::set_var("QSG_RHI_BACKEND", "opengl");

    let mut app = QGuiApplication::new();

    // Force that we want OpenGLES
    ffi::force_surface_format();

    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/kdab/servo/qml/main.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
