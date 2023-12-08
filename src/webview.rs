#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QUrl, url)]
        type ServoWebView = super::QServoWebViewRust;
    }

    impl cxx_qt::Constructor<()> for ServoWebView {}
    impl cxx_qt::Threading for ServoWebView {}
}

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QUrl;
use servo::compositing::windowing::EmbedderEvent;
use servo::servo_url::ServoUrl;
use servo::Servo;
use std::rc::Rc;

use crate::browser::QServoBrowser;
use crate::embedder::QServoEmbedder;
use crate::events_loop::QServoEventsLoopWaker;
use crate::window::QServoWindow;

#[derive(Default)]
pub struct QServoWebViewRust {
    browser: QServoBrowser,
    servo: Option<Servo<QServoWindow>>,
    url: QUrl,
}

impl QServoWebViewRust {
    fn as_servo_url(&self) -> Result<ServoUrl, url::ParseError> {
        Ok(ServoUrl::from_url(url::Url::try_from(&self.url)?))
    }
}

impl qobject::ServoWebView {
    fn handle_events(mut self: core::pin::Pin<&mut Self>) {
        if self.servo.is_none() {
            return;
        }

        // Browser process servo events
        let servo_events = self
            .as_mut()
            .rust_mut()
            .servo
            .as_mut()
            .unwrap()
            .get_events();
        self.as_mut()
            .rust_mut()
            .browser
            .handle_servo_events(servo_events);

        // Servo process browser events
        let browser_events = self.as_mut().rust_mut().browser.get_events();
        self.as_mut()
            .rust_mut()
            .servo
            .as_mut()
            .unwrap()
            .handle_events(browser_events);

        // Queue again in the next event loop
        self.qt_thread()
            .queue(|qobject| {
                qobject.handle_events();
            })
            .unwrap();
    }
}

impl cxx_qt::Initialize for qobject::ServoWebView {
    fn initialize(mut self: core::pin::Pin<&mut Self>) {
        self.as_mut().qt_thread().queue(|mut qobject| {
            let event_loop_waker = Box::new(QServoEventsLoopWaker::default());
            let embedder = Box::new(QServoEmbedder::new(event_loop_waker));
            let window = Rc::new(QServoWindow::from_qwindow());
            let user_agent = None;
            // The in-process interface to Servo.
            //
            // It does everything necessary to render the web, primarily
            // orchestrating the interaction between JavaScript, CSS layout,
            // rendering, and the client window.
            //
            // Clients create a `Servo` instance for a given reference-counted type
            // implementing `WindowMethods`, which is the bridge to whatever
            // application Servo is embedded in. Clients then create an event
            // loop to pump messages between the embedding application and
            // various browser components.
            let mut servo_data = Servo::new(embedder, window, user_agent);

            // Create an initial state
            servo_data
                .servo
                .handle_events(vec![EmbedderEvent::NewBrowser(
                    qobject.rust().as_servo_url().unwrap().into(),
                    servo_data.browser_id,
                )]);
            servo_data.servo.setup_logging();

            qobject.as_mut().rust_mut().servo = Some(servo_data.servo);

            // Start process events on the Qt event loop
            qobject.as_mut()
                .qt_thread()
                .queue(|qobject| {
                    qobject.handle_events();
                })
                .unwrap();

            // Tell servo when the URL has been updated
            qobject.on_url_changed(|mut qobject| {
                // FIXME: assumes there is an ID
                let browser_id = qobject.rust().browser.browser_id().unwrap();
                let servo_url = qobject.rust().as_servo_url().unwrap();
                qobject
                    .as_mut()
                    .rust_mut()
                    .browser
                    .push_event(EmbedderEvent::LoadUrl(browser_id, servo_url));
            })
            .release();
        }).unwrap();
    }
}
