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

    /// This enum specifies why the focus changed. It will be passed through QWidget::setFocus
    /// and can be retrieved in the QFocusEvent sent to the widget upon focus change.
    #[namespace = "Qt"]
    #[repr(i32)]
    enum FocusReason {
        /// A mouse action occurred.
        MouseFocusReason,
        /// The Tab key was pressed.
        TabFocusReason,
        /// A Backtab occurred. The input for this may include the Shift or Control keys; e.g. Shift+Tab.
        BacktabFocusReason,
        /// The window system made this window either active or inactive.
        ActiveWindowFocusReason,
        /// The application opened/closed a pop-up that grabbed/released the keyboard focus.
        PopupFocusReason,
        /// The user typed a label's buddy shortcut
        ShortcutFocusReason,
        /// The menu bar took focus.
        MenuBarFocusReason,
        /// Another reason, usually application-specific.
        OtherFocusReason,
    }

    #[namespace = "Qt"]
    unsafe extern "C++" {
        type FocusReason;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QQuickFramebufferObject"]
        #[qml_element]
        #[qproperty(QUrl, favicon_url)]
        #[qproperty(bool, loading)]
        #[qproperty(bool, navigation_allowed)]
        #[qproperty(QString, title)]
        #[qproperty(QUrl, url)]
        type ServoWebView = super::QServoWebViewRust;

        #[cxx_override]
        #[cxx_name = "createRenderer"]
        fn create_renderer(self: &ServoWebView) -> *mut QQuickFramebufferObjectRenderer;

        #[inherit]
        #[cxx_name = "forceActiveFocus"]
        fn force_active_focus(self: Pin<&mut ServoWebView>, reason: FocusReason);

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

        #[qsignal]
        fn blocked_navigation_request(self: Pin<&mut ServoWebView>, blocked_url: QUrl);
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qpoint.h");
        type QPoint = cxx_qt_lib::QPoint;
        include!("cxx-qt-lib/qpointf.h");
        type QPointF = cxx_qt_lib::QPointF;

        type QWheelEvent;

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

    unsafe extern "C++" {
        type QKeyEvent;

        #[cxx_name = "isAutoRepeat"]
        fn is_auto_repeat(&self) -> bool;

        fn key(&self) -> i32;

        fn text(&self) -> QString;
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
        #[cxx_name = "keyPressEvent"]
        #[cxx_override]
        unsafe fn key_press_event(self: Pin<&mut ServoWebView>, event: *mut QKeyEvent);

        #[cxx_name = "keyReleaseEvent"]
        #[cxx_override]
        unsafe fn key_release_event(self: Pin<&mut ServoWebView>, event: *mut QKeyEvent);

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
use qobject::{FocusReason, QEventPointState, QMouseEventButton};
use servo::{
    compositing::windowing::{EmbedderEvent, MouseWindowEvent},
    keyboard_types::{Code, Key, KeyState, KeyboardEvent, Location, Modifiers},
    script_traits::{MouseButton, TouchEventType, TouchId},
};
use std::str::FromStr;

use crate::renderer::qobject::QServoRenderer;

impl qobject::QTouchEvent {
    fn point_count(&self) -> isize {
        qobject::qtouchevent_point_count(self)
    }

    fn point(self: Pin<&mut Self>, index: isize) -> &qobject::QEventPoint {
        qobject::qtouchevent_point(self, index)
    }
}

fn get_servo_code_from_scancode(code: i32) -> Code {
    // TODO: Map more codes
    use servo::keyboard_types::Code::*;
    match code {
        0x01000000 => Escape,
        0x01000001 => Tab,
        // 0x01000002
        0x01000003 => Backspace,
        0x01000004 => Enter,
        0x01000005 => NumpadEnter,
        0x01000006 => Insert,
        0x01000007 => Delete,
        // ...
        0x01000010 => Home,
        0x01000011 => End,
        0x01000012 => ArrowLeft,
        0x01000013 => ArrowUp,
        0x01000014 => ArrowRight,
        0x01000015 => ArrowDown,
        0x01000016 => PageUp,
        0x01000017 => PageDown,
        // Shift
        0x01000020 => ShiftLeft,
        // ...
        0x01000030 => F1,
        0x01000031 => F2,
        0x01000032 => F3,
        0x01000033 => F4,
        0x01000034 => F5,
        0x01000035 => F6,
        0x01000036 => F7,
        0x01000037 => F8,
        0x01000038 => F9,
        0x01000039 => F10,
        0x0100003a => F11,
        0x0100003b => F12,
        // ...
        0x20 => Space,
        // ...
        0x2c => Comma,
        // 0x2d
        0x2e => Period,
        0x2f => Slash,
        0x30 => Digit0,
        0x31 => Digit1,
        0x32 => Digit2,
        0x33 => Digit3,
        0x34 => Digit4,
        0x35 => Digit5,
        0x36 => Digit6,
        0x37 => Digit7,
        0x38 => Digit8,
        0x39 => Digit9,
        // 0x3a
        0x3b => Semicolon,
        // ...
        0x41 => KeyA,
        0x42 => KeyB,
        0x43 => KeyC,
        0x44 => KeyD,
        0x45 => KeyE,
        0x46 => KeyF,
        0x47 => KeyG,
        0x48 => KeyH,
        0x49 => KeyI,
        0x4a => KeyJ,
        0x4b => KeyK,
        0x4c => KeyL,
        0x4d => KeyM,
        0x4e => KeyN,
        0x4f => KeyO,
        0x50 => KeyP,
        0x51 => KeyQ,
        0x52 => KeyR,
        0x53 => KeyS,
        0x54 => KeyT,
        0x55 => KeyU,
        0x56 => KeyV,
        0x57 => KeyW,
        0x58 => KeyX,
        0x59 => KeyY,
        0x5a => KeyZ,
        0x5b => BracketLeft,
        0x5c => Backslash,
        0x5d => BracketRight,
        // ...
        // QuoteLeft
        0x60 => Quote,
        // ...
        _ => Unidentified,
    }
}

pub struct QServoWebViewRust {
    favicon_url: QUrl,
    loading: bool,
    title: QString,
    url: QUrl,
    pub(crate) events: Vec<EmbedderEvent>,
    press_position: Option<QPointF>,
    navigation_allowed: bool,
}

impl Default for QServoWebViewRust {
    fn default() -> Self {
        Self {
            favicon_url: QUrl::default(),
            loading: false,
            title: QString::default(),
            url: QUrl::default(),
            events: vec![],
            press_position: None,
            navigation_allowed: true,
        }
    }
}

impl qobject::ServoWebView {
    fn create_renderer(&self) -> *mut qobject::QQuickFramebufferObjectRenderer {
        QServoRenderer::new().into_raw() as *mut qobject::QQuickFramebufferObjectRenderer
    }

    fn key_event(mut self: Pin<&mut Self>, event: *mut qobject::QKeyEvent, state: KeyState) {
        if let Some(event) = unsafe { event.as_ref() } {
            let keyboard_event = KeyboardEvent {
                state,
                key: Key::from_str(&String::from(&event.text()))
                    .unwrap_or_else(|_| Key::Unidentified),
                code: get_servo_code_from_scancode(event.key()),
                repeat: event.is_auto_repeat(),
                // TODO: handle these correctly
                location: Location::Standard,
                modifiers: Modifiers::empty(),
                is_composing: false,
            };
            self.as_mut()
                .rust_mut()
                .events
                .push(EmbedderEvent::Keyboard(keyboard_event));

            self.as_mut().update();
        }
    }

    fn key_press_event(self: Pin<&mut Self>, event: *mut qobject::QKeyEvent) {
        self.key_event(event, KeyState::Down);
    }

    fn key_release_event(self: Pin<&mut Self>, event: *mut qobject::QKeyEvent) {
        self.key_event(event, KeyState::Up);
    }

    fn mouse_move_event(mut self: Pin<&mut Self>, event: *mut qobject::QMouseEvent) {
        if let Some(event) = unsafe { event.as_ref() } {
            let event_position = event.position();
            let position = Point2D::new(event_position.x() as f32, event_position.y() as f32);
            self.as_mut()
                .rust_mut()
                .events
                .push(EmbedderEvent::MouseWindowMoveEventClass(position));

            self.as_mut().update();
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

            // Ensure we have focus so that we receive key events
            self.as_mut()
                .force_active_focus(FocusReason::MouseFocusReason);

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

            if let Some(press_position) = self.as_mut().rust_mut().press_position.take() {
                // If the press position is close to the release then assume a click
                let diff = press_position - event_position;
                let dist = (diff.x().powf(2.0) + diff.y().powf(2.0)).sqrt();
                let dpi_factor = 1.0; // TODO: read DPI settings
                if dist < 10.0 * dpi_factor {
                    self.as_mut()
                        .rust_mut()
                        .events
                        .push(EmbedderEvent::MouseWindowEventClass(
                            MouseWindowEvent::Click(button, position),
                        ));
                }
            }

            // Ensure we have focus so that we receive key events
            self.as_mut()
                .force_active_focus(FocusReason::MouseFocusReason);

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

            // Ensure we have focus so that we receive key events
            self.as_mut()
                .force_active_focus(FocusReason::MouseFocusReason);

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
