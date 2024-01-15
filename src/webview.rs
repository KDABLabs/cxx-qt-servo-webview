// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

#[cxx_qt::bridge(cxx_file_stem = "servowebview")]
pub(crate) mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;

        include!(<QQuickFramebufferObject>);

        include!("helpers.h");
        // TODO: describe this with a custom type
        type QQuickFramebufferObjectRenderer;

        #[cxx_name = "setMirrorVertically"]
        fn set_mirror_vertically(self: Pin<&mut ServoWebView>, enable: bool);
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QQuickFramebufferObject"]
        #[qml_element]
        #[qproperty(QUrl, favicon_url)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, title)]
        #[qproperty(QUrl, url)]
        type ServoWebView = super::QServoWebViewRust;

        #[cxx_override]
        #[cxx_name = "createRenderer"]
        fn create_renderer(self: &ServoWebView) -> *mut QQuickFramebufferObjectRenderer;

        #[inherit]
        fn update(self: Pin<&mut ServoWebView>);
    }

    impl cxx_qt::Constructor<()> for ServoWebView {}
    impl cxx_qt::Threading for ServoWebView {}
}

use core::pin::Pin;
use cxx_qt_lib::{QString, QUrl};

use crate::renderer::qobject::QServoRenderer;

#[derive(Default)]
pub struct QServoWebViewRust {
    favicon_url: QUrl,
    loading: bool,
    title: QString,
    url: QUrl,
}

impl qobject::ServoWebView {
    fn create_renderer(&self) -> *mut qobject::QQuickFramebufferObjectRenderer {
        QServoRenderer::new().into_raw() as *mut qobject::QQuickFramebufferObjectRenderer
    }
}

impl cxx_qt::Initialize for qobject::ServoWebView {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().set_mirror_vertically(true);

        // When the URL changes trigger QQuickFramebufferObject::update
        // which then triggers QQuickFramebufferObject::Renderer::synchronize
        self.on_url_changed(|qobject| {
            qobject.update();
        })
        .release();
    }
}
