// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    rc::Rc,
    sync::mpsc::{Receiver, SyncSender},
};

use cxx_qt::CxxQtThread;
use cxx_qt_lib::{QString, QUrl};
use servo::{
    compositing::windowing::{EmbedderEvent, WindowMethods},
    embedder_traits::EventLoopWaker,
    euclid::Size2D,
    servo_url::ServoUrl,
    style_traits::DevicePixel,
    Servo, TopLevelBrowsingContextId,
};
use surfman::chains::SwapChainAPI;
use surfman::{Connection, Surface};

use crate::{
    browser::QServoBrowser, embedder::QServoEmbedder, events_loop::QServoEventsLoopWaker,
    webview::qobject::ServoWebView, windowheadless::QServoWindowHeadless,
};

// #[derive(Debug)]
pub(crate) enum QServoMessage {
    RawEmbeddedEvent(EmbedderEvent),
    Resize(Size2D<i32, DevicePixel>),
    Url(ServoUrl),
    Heartbeat(SyncSender<()>),
    BorrowSurface(SyncSender<Option<Surface>>, Receiver<Option<Surface>>),
    Quit,
}

unsafe impl Send for QServoMessage {}

pub(crate) struct QServoThread {
    browser: QServoBrowser,
    browser_id: TopLevelBrowsingContextId,
    servo: Servo<QServoWindowHeadless>,
    receiver: Receiver<QServoMessage>,
    qt_thread: CxxQtThread<ServoWebView>,
}

impl QServoThread {
    pub(crate) fn new(
        receiver: Receiver<QServoMessage>,
        qt_thread: CxxQtThread<ServoWebView>,
        connection: Connection,
    ) -> Self {
        let event_loop_waker = QServoEventsLoopWaker::new(qt_thread.clone());
        let embedder = Box::new(QServoEmbedder::new(event_loop_waker.clone_box()));

        let window = Rc::new(QServoWindowHeadless::new(Size2D::new(400, 400), connection));
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
        let servo_data = Servo::new(
            embedder,
            window.clone(),
            user_agent,
            servo::compositing::CompositeTarget::Window,
        );

        // Enable logging and store servo instance
        servo_data.servo.setup_logging();

        // Initialise servo
        event_loop_waker.wake();

        Self {
            browser_id: servo_data.browser_id,
            servo: servo_data.servo,
            browser: QServoBrowser::default(),
            receiver,
            qt_thread,
        }
    }

    pub(crate) fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                QServoMessage::RawEmbeddedEvent(event) => {
                    self.browser.push_event(event);
                }
                QServoMessage::Resize(size) => {
                    let surfman = self.servo.window().rendering_context();
                    surfman
                        .resize(size.to_untyped().to_i32())
                        .expect("Failed to resize");
                    self.browser.push_event(EmbedderEvent::Resize);
                }
                QServoMessage::Url(url) => {
                    // Open a new browser or load the url
                    if let Some(webview_id) = self.browser.webview_id() {
                        self.browser
                            .push_event(EmbedderEvent::LoadUrl(webview_id, url));
                    } else {
                        self.browser
                            .push_event(EmbedderEvent::NewWebView(url, self.browser_id));
                    }
                }
                QServoMessage::BorrowSurface(sender, receiver) => {
                    let surfman = self.servo.window().rendering_context();
                    let swap_chain = surfman.swap_chain().unwrap();

                    let surface = swap_chain.take_surface();

                    println!("sending surface: {}", surface.is_some());
                    sender.send(surface).unwrap();

                    println!("waiting for surface return");
                    let surface = receiver.recv().unwrap();

                    println!("returned surface, recycling");
                    if let Some(surface) = surface {
                        swap_chain.recycle_surface(surface);
                    }
                }
                QServoMessage::Heartbeat(sender) => {
                    // Browser process servo events
                    let servo_events = self.servo.get_events();
                    let response = self.browser.handle_servo_events(servo_events);

                    // Handle the responses from browser events to Qt
                    self.qt_thread
                        .queue(move |mut webview| {
                            if let Some(title) = response.title {
                                webview.as_mut().set_title(QString::from(&title));
                            }
                            if let Some(loading) = response.loading {
                                webview.as_mut().set_loading(loading);
                            }
                            if let Some(favicon_url) = response.favicon_url {
                                webview.as_mut().set_favicon_url(QUrl::from(&favicon_url));
                            }
                            if let Some(url) = response.url {
                                webview.as_mut().set_url(QUrl::from(&url));
                            }
                        })
                        .unwrap();

                    // Present when required
                    let need_present = response.present.unwrap_or(false);

                    // Servo process browser events
                    let browser_events = self.browser.get_events();
                    let need_resize = self.servo.handle_events(browser_events);

                    // If we have resized then we need to force a repaint synchornously
                    //
                    // This is the same as Present::Immediate in servoshell
                    // Resizes are unusual in that we need to repaint synchronously.
                    // TODO(servo#30049) can we replace this with the simpler Servo::recomposite?
                    if need_resize {
                        println!("repaint_synchronously");
                        self.servo.repaint_synchronously();
                    } else {
                        // If we aren't resizing then ensure the compositor is ready
                        // otherwise after a resize we can have an empty frame
                        self.servo.recomposite();
                    }

                    // Instead we do this after recycle_surface as this avoids issues
                    // with the surface becoming empty after resize
                    // Present if we resized or need to present
                    if need_present || need_resize {
                        println!("present!");
                        self.servo.present();
                    }

                    // Indicate that we have completed the heartbeat
                    sender.send(()).unwrap();

                    println!("heartbeat!");
                }
                QServoMessage::Quit => break,
            }
        }

        println!("quiting!");
        self.servo.handle_events(vec![EmbedderEvent::Quit]);
    }
}
