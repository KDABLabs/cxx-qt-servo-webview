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
    BrowserId, Servo,
};
use surfman::chains::SwapChainAPI;
use surfman::{Connection, Surface};

use crate::{
    browser::QServoBrowser, embedder::QServoEmbedder, events_loop::QServoEventsLoopWaker,
    webview::qobject::ServoWebView, windowheadless::QServoWindowHeadless,
};

#[derive(Debug)]
pub(crate) enum QServoMessage {
    Url(ServoUrl),
    Heartbeat,
    BorrowSurface(SyncSender<Option<Surface>>, Receiver<Option<Surface>>),
    Quit,
}

pub(crate) struct QServoThread {
    browser: QServoBrowser,
    browser_id: BrowserId,
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
        let servo_data = Servo::new(embedder, window.clone(), user_agent);

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
                QServoMessage::Url(url) => {
                    // Open a new browser or load the url
                    if let Some(browser_id) = self.browser.browser_id() {
                        self.browser
                            .push_event(EmbedderEvent::LoadUrl(browser_id, url));
                    } else {
                        self.browser
                            .push_event(EmbedderEvent::NewBrowser(url, self.browser_id));
                    }
                }
                QServoMessage::BorrowSurface(sender, receiver) => {
                    let surfman = self.servo.window().webrender_surfman();
                    let swap_chain = surfman.swap_chain().unwrap();

                    // TODO: when do we need to recomposite?
                    println!("recomposite!");
                    self.servo.recomposite();

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
                QServoMessage::Heartbeat => {
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
                        })
                        .unwrap();

                    // Present when required
                    if let Some(present) = response.present {
                        if present {
                            println!("present!");
                            self.servo.present();
                        }
                    }

                    // Servo process browser events
                    let browser_events = self.browser.get_events();
                    self.servo.handle_events(browser_events);

                    println!("heartbeat!");
                }
                QServoMessage::Quit => break,
            }
        }

        println!("quiting!");
        self.servo.handle_events(vec![EmbedderEvent::Quit]);
    }
}
