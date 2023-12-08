#[cxx_qt::bridge]
pub(crate) mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QUrl, favicon_url)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, title)]
        #[qproperty(QUrl, url)]
        type ServoWebView = super::QServoWebViewRust;
    }

    impl cxx_qt::Constructor<()> for ServoWebView {}
    impl cxx_qt::Threading for ServoWebView {}
}

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QUrl};
use servo::compositing::windowing::EmbedderEvent;
use servo::embedder_traits::EventLoopWaker;
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
    favicon_url: QUrl,
    loading: bool,
    servo: Option<Servo<QServoWindow>>,
    title: QString,
    url: QUrl,
}

impl QServoWebViewRust {
    fn as_servo_url(&self) -> Result<ServoUrl, url::ParseError> {
        Ok(ServoUrl::from_url(url::Url::try_from(&self.url)?))
    }
}

impl qobject::ServoWebView {
    pub(crate) fn handle_events(mut self: core::pin::Pin<&mut Self>) {
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
        let response = self
            .as_mut()
            .rust_mut()
            .browser
            .handle_servo_events(servo_events);

        // Handle the responses from browser events to Qt
        if let Some(title) = response.title {
            self.as_mut().set_title(QString::from(&title));
        }
        if let Some(loading) = response.loading {
            self.as_mut().set_loading(loading);
        }
        if let Some(favicon_url) = response.favicon_url {
            self.as_mut().set_favicon_url(QUrl::from(&favicon_url));
        }
        if let Some(_preset) = response.present {
            // TODO: tell Qt to paint if present is ready
            // self.as_mut().rust_mut().servo.as_mut().unwrap().recomposite();
            // self.as_mut().rust_mut().servo.as_mut().unwrap().present();
        }

        // Servo process browser events
        let browser_events = self.as_mut().rust_mut().browser.get_events();
        self.as_mut()
            .rust_mut()
            .servo
            .as_mut()
            .unwrap()
            .handle_events(browser_events);
    }
}

impl cxx_qt::Initialize for qobject::ServoWebView {
    fn initialize(mut self: core::pin::Pin<&mut Self>) {
        self.as_mut()
            .qt_thread()
            .queue(|mut qobject| {
                let event_loop_waker = QServoEventsLoopWaker::new(qobject.as_mut().qt_thread());
                let embedder = Box::new(QServoEmbedder::new(event_loop_waker.clone_box()));
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
                let servo_data = Servo::new(embedder, window.clone(), user_agent);

                // Create an initial state
                let event = EmbedderEvent::NewBrowser(
                    qobject.rust().as_servo_url().unwrap().into(),
                    servo_data.browser_id,
                );
                qobject.as_mut().rust_mut().browser.push_event(event);
                event_loop_waker.wake();

                // Enable logging and store servo instance
                servo_data.servo.setup_logging();
                qobject.as_mut().rust_mut().servo = Some(servo_data.servo);

                // Tell servo when the URL has been updated
                qobject
                    .on_url_changed(move |mut qobject| {
                        // The browser id comes from the initial state
                        let browser_id = qobject.rust().browser.browser_id().unwrap();
                        let servo_url = qobject.rust().as_servo_url().unwrap();
                        qobject
                            .as_mut()
                            .rust_mut()
                            .browser
                            .push_event(EmbedderEvent::LoadUrl(browser_id, servo_url));
                        event_loop_waker.wake();

                        // Clear any favicon
                        qobject.as_mut().set_favicon_url(QUrl::default());
                    })
                    .release();
            })
            .unwrap();
    }
}
