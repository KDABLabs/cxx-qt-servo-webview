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
        #[cxx_name = "setAcceptedMouseButtons"]
        fn set_accepted_mouse_buttons(self: Pin<&mut ServoWebView>, buttons: QMouseEventButton);

        #[inherit]
        #[cxx_name = "setAcceptTouchEvents"]
        fn set_accept_touch_events(self: Pin<&mut ServoWebView>, enabled: bool);

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

    #[repr(u8)]
    enum QEventPointState {
        Unknown = 0x00,
        Stationary = 0x04,
        Pressed = 0x01,
        Updated = 0x02,
        Released = 0x08,
    }

    unsafe extern "C++" {
        type QEventPointState;
    }

    unsafe extern "C++" {
        type QEventPoint;

        fn id(&self) -> i32;
        fn position(&self) -> QPointF;
        fn state(&self) -> QEventPointState;
    }

    #[repr(i32)]
    enum QMouseEventButton {
        NoButton = 0,
        AllButtons = 0x07ffffff,
        LeftButton = 0x01,
        RightButton = 0x02,
        MiddleButton = 0x04,
    }

    unsafe extern "C++" {
        type QMouseEventButton;
    }

    unsafe extern "C++" {
        type QMouseEvent;

        fn button(&self) -> QMouseEventButton;
        fn position(&self) -> QPointF;
    }

    unsafe extern "C++" {
        type QTouchEvent;

        #[cxx_name = "qTouchEventPointCount"]
        fn qtouchevent_point_count(ptr: &QTouchEvent) -> isize;

        #[cxx_name = "qTouchEventPoint"]
        fn qtouchevent_point(ptr: Pin<&mut QTouchEvent>, i: isize) -> &QEventPoint;
    }

    unsafe extern "RustQt" {
        #[cxx_override]
        #[cxx_name = "mouseMoveEvent"]
        unsafe fn mouse_move_event(self: Pin<&mut ServoWebView>, event: *mut QMouseEvent);

        #[cxx_override]
        #[cxx_name = "mousePressEvent"]
        unsafe fn mouse_press_event(self: Pin<&mut ServoWebView>, event: *mut QMouseEvent);

        #[cxx_override]
        #[cxx_name = "mouseReleaseEvent"]
        unsafe fn mouse_release_event(self: Pin<&mut ServoWebView>, event: *mut QMouseEvent);

        #[cxx_override]
        #[cxx_name = "touchEvent"]
        unsafe fn touch_event(self: Pin<&mut ServoWebView>, event: *mut QTouchEvent);

        #[cxx_override]
        #[cxx_name = "wheelEvent"]
        unsafe fn wheel_event(self: Pin<&mut ServoWebView>, event: *mut QWheelEvent);
    }

    impl cxx_qt::Constructor<()> for ServoWebView {}
    impl cxx_qt::Threading for ServoWebView {}
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QPointF, QString, QUrl};
use euclid::Point2D;
use qobject::{QEventPointState, QMouseEventButton};
use servo::{
    compositing::windowing::{EmbedderEvent, MouseWindowEvent},
    script_traits::{MouseButton, TouchEventType, TouchId},
};

use crate::renderer::qobject::QServoRenderer;

impl qobject::QTouchEvent {
    fn point_count(&self) -> isize {
        qobject::qtouchevent_point_count(self)
    }

    fn point(self: Pin<&mut Self>, index: isize) -> &qobject::QEventPoint {
        qobject::qtouchevent_point(self, index)
    }
}

#[derive(Default)]
pub struct QServoWebViewRust {
    favicon_url: QUrl,
    loading: bool,
    title: QString,
    url: QUrl,
    pub(crate) events: Vec<EmbedderEvent>,
    press_position: Option<QPointF>,
}

impl qobject::ServoWebView {
    fn create_renderer(&self) -> *mut qobject::QQuickFramebufferObjectRenderer {
        QServoRenderer::new().into_raw() as *mut qobject::QQuickFramebufferObjectRenderer
    }

    fn mouse_move_event(mut self: Pin<&mut Self>, event: *mut qobject::QMouseEvent) {
        if let Some(event) = unsafe { event.as_ref() } {
            let event_position = event.position();
            let position = Point2D::new(event_position.x() as f32, event_position.y() as f32);
            self.as_mut()
                .rust_mut()
                .events
                .push(EmbedderEvent::MouseWindowMoveEventClass(position));
        }
    }

    fn mouse_press_event(mut self: Pin<&mut Self>, event: *mut qobject::QMouseEvent) {
        if let Some(event) = unsafe { event.as_ref() } {
            let event_position = event.position();
            let position = Point2D::new(event_position.x() as f32, event_position.y() as f32);
            let button = match event.button() {
                QMouseEventButton::LeftButton => MouseButton::Left,
                QMouseEventButton::RightButton => MouseButton::Right,
                QMouseEventButton::MiddleButton => MouseButton::Middle,
                _others => return,
            };
            self.as_mut()
                .rust_mut()
                .events
                .push(EmbedderEvent::MouseWindowEventClass(
                    MouseWindowEvent::MouseDown(button, position),
                ));

            // Store the event position so we can detect clicks
            self.as_mut().rust_mut().press_position = Some(event_position);

            self.as_mut().update();
        }
    }

    fn mouse_release_event(mut self: Pin<&mut Self>, event: *mut qobject::QMouseEvent) {
        if let Some(event) = unsafe { event.as_ref() } {
            let event_position = event.position();
            let position = Point2D::new(event_position.x() as f32, event_position.y() as f32);
            let button = match event.button() {
                QMouseEventButton::LeftButton => MouseButton::Left,
                QMouseEventButton::RightButton => MouseButton::Right,
                QMouseEventButton::MiddleButton => MouseButton::Middle,
                _others => return,
            };
            self.as_mut()
                .rust_mut()
                .events
                .push(EmbedderEvent::MouseWindowEventClass(
                    MouseWindowEvent::MouseUp(button, position),
                ));

            // If the press position is the same as the release then assume a click
            if let Some(press_position) = self.as_mut().rust_mut().press_position.take() {
                if press_position == event_position {
                    self.as_mut()
                        .rust_mut()
                        .events
                        .push(EmbedderEvent::MouseWindowEventClass(
                            MouseWindowEvent::Click(button, position),
                        ));
                }
            }

            self.as_mut().update();
        }
    }

    fn touch_event(mut self: Pin<&mut Self>, event: *mut qobject::QTouchEvent) {
        if let Some(event) = unsafe { event.as_mut() } {
            let mut event = unsafe { Pin::new_unchecked(event) };
            let points = event.as_ref().point_count();
            if points == 0 {
                // Empty points in Qt means that touch events have been cancelled
                // emulate cancelling 4 ids as we have no id here
                for id in 0..3 {
                    self.as_mut().rust_mut().events.push(EmbedderEvent::Touch(
                        TouchEventType::Cancel,
                        TouchId(id),
                        Point2D::new(0 as f32, 0 as f32),
                    ));
                }
            } else {
                for i in 0..points {
                    let point = event.as_mut().point(i);
                    let position = point.position();
                    let position = Point2D::new(position.x() as f32, position.y() as f32);
                    let phase = match point.state() {
                        QEventPointState::Unknown | QEventPointState::Stationary => continue,
                        QEventPointState::Pressed => TouchEventType::Down,
                        QEventPointState::Updated => TouchEventType::Move,
                        QEventPointState::Released => TouchEventType::Up,
                        _others => continue,
                    };
                    self.as_mut().rust_mut().events.push(EmbedderEvent::Touch(
                        phase,
                        TouchId(point.id()),
                        position,
                    ));
                }
            }

            self.as_mut().update();
        }
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
                    // TODO: consider scrolling phase
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
        // TODO: we don't support QFlags so just enable everything for now
        self.as_mut()
            .set_accepted_mouse_buttons(QMouseEventButton::AllButtons);

        self.as_mut().set_accept_touch_events(true);
        self.as_mut().set_mirror_vertically(true);

        // When the URL changes trigger QQuickFramebufferObject::update
        // which then triggers QQuickFramebufferObject::Renderer::synchronize
        self.on_url_changed(|qobject| {
            qobject.update();
        })
        .release();
    }
}
