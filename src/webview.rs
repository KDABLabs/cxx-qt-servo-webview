// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

#[cxx_qt::bridge(cxx_file_stem = "servowebview")]
pub(crate) mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qsizef.h");
        type QSizeF = cxx_qt_lib::QSizeF;

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
        fn size(self: &ServoWebView) -> QSizeF;

        #[inherit]
        fn update(self: Pin<&mut ServoWebView>);
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qpoint.h");
        type QPoint = cxx_qt_lib::QPoint;
        include!("cxx-qt-lib/qpointf.h");
        type QPointF = cxx_qt_lib::QPointF;

        type QWheelEvent;

        #[cxx_name = "angleDelta"]
        fn angle_delta(self: &QWheelEvent) -> QPoint;

        #[cxx_name = "pixelDelta"]
        fn pixel_delta(self: &QWheelEvent) -> QPoint;

        fn position(self: &QWheelEvent) -> QPointF;
    }

    unsafe extern "RustQt" {
        #[cxx_override]
        #[cxx_name = "wheelEvent"]
        unsafe fn wheel_event(self: Pin<&mut ServoWebView>, event: *mut QWheelEvent);
    }

    impl cxx_qt::Constructor<()> for ServoWebView {}
    impl cxx_qt::Threading for ServoWebView {}
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QUrl};
use servo::compositing::windowing::EmbedderEvent;

use crate::renderer::qobject::QServoRenderer;

#[derive(Default)]
pub struct QServoWebViewRust {
    favicon_url: QUrl,
    loading: bool,
    title: QString,
    url: QUrl,
    pub(crate) events: Vec<EmbedderEvent>,
}

impl qobject::ServoWebView {
    fn create_renderer(&self) -> *mut qobject::QQuickFramebufferObjectRenderer {
        QServoRenderer::new().into_raw() as *mut qobject::QQuickFramebufferObjectRenderer
    }

    fn wheel_event(mut self: Pin<&mut Self>, event: *mut qobject::QWheelEvent) {
        if let Some(event) = unsafe { event.as_ref() } {
            // TODO: consider angle_delta
            // https://doc.qt.io/qt-6/qwheelevent.html#angleDelta
            let pixel_delta = event.pixel_delta();
            let position = event.position();

            self.as_mut().rust_mut().events.push(EmbedderEvent::Wheel(
                servo::script_traits::WheelDelta {
                    x: pixel_delta.x() as f64,
                    y: pixel_delta.y() as f64,
                    z: 0.0,
                    mode: servo::script_traits::WheelMode::DeltaPixel,
                },
                euclid::Point2D::new(position.x() as f32, position.y() as f32),
            ));

            // Scroll events snap to the major axis of movement, with vertical
            // preferred over horizontal.
            let (dx, dy) = if pixel_delta.y().abs() > pixel_delta.x().abs() {
                (0.0, pixel_delta.y() as f32)
            } else {
                (pixel_delta.x() as f32, 0.0)
            };

            let scroll_location =
                servo::webrender_api::ScrollLocation::Delta(euclid::Vector2D::new(dx, dy));

            self.as_mut().rust_mut().events.push(
                servo::compositing::windowing::EmbedderEvent::Scroll(
                    scroll_location,
                    euclid::Point2D::new(position.x() as i32, position.y() as i32),
                    // TODO: consier scrolling phase
                    // https://doc.qt.io/qt-6/qwheelevent.html#phase
                    servo::script_traits::TouchEventType::Move,
                ),
            );

            self.as_mut().update();
        }
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
