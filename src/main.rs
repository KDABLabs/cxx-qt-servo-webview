use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

mod browser;
mod embedder;
mod events_loop;
mod webview;
mod window;

fn main() {
    std::env::set_var("QSG_RHI_BACKEND", "opengl");

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/kdab/servo/qml/main.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
