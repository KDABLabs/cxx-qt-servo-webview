// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

#[cxx_qt::bridge(cxx_file_stem = "servorenderer")]
pub(crate) mod qobject {
    unsafe extern "C++" {
        include!(<QQuickFramebufferObject>);
        type QQuickFramebufferObject;
    }

    unsafe extern "C++" {
        include!("helpers.h");

        #[cxx_name = "constructUniquePtr"]
        fn qservo_renderer_unique_ptr() -> UniquePtr<QServoRenderer>;
    }

    unsafe extern "C++" {
        include!(<QOpenGLFramebufferObject>);
        type QOpenGLFramebufferObject;

        include!("helpers.h");
        #[cxx_name = "blitFramebuffer"]
        unsafe fn blit_framebuffer(
            target: *mut QOpenGLFramebufferObject,
            source: *mut QOpenGLFramebufferObject,
        );

        include!("cxx-qt-lib/qsize.h");
        type QSize = cxx_qt_lib::QSize;

        #[cxx_name = "fboFromTexture"]
        fn fbo_from_texture(
            texture_id: u32,
            texture_target: u32,
            size: QSize,
        ) -> *mut QOpenGLFramebufferObject;
    }

    unsafe extern "RustQt" {
        #[base = "QQuickFramebufferObject::Renderer"]
        type QServoRenderer = super::QServoRendererRust;

        #[inherit]
        #[rust_name = "framebuffer_object"]
        fn framebufferObject(self: &QServoRenderer) -> *mut QOpenGLFramebufferObject;

        #[cxx_override]
        fn render(self: Pin<&mut QServoRenderer>);

        #[cxx_override]
        unsafe fn synchronize(self: Pin<&mut QServoRenderer>, item: *mut QQuickFramebufferObject);
    }

    impl cxx_qt::Constructor<()> for QServoRenderer {}
}

use crate::{
    browser::QServoBrowser,
    embedder::QServoEmbedder,
    events_loop::QServoEventsLoopWaker,
    servothread::{QServoMessage, QServoThread},
    webview::qobject::ServoWebView,
    windowheadless::QServoWindowHeadless,
};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QSize, QString, QUrl};
use servo::{
    compositing::windowing::{EmbedderEvent, WindowMethods},
    embedder_traits::EventLoopWaker,
    euclid::Size2D,
    servo_url::ServoUrl,
    BrowserId, Servo,
};
use std::{
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};
use surfman::{Context, Device};

#[derive(Default)]
pub struct QServoRendererRust {
    // browser: QServoBrowser,
    // browser_id: Option<BrowserId>,
    // // need_preset: bool,
    // servo: Option<Servo<QServoWindowHeadless>>,
    url: QUrl,
    sender: Option<Sender<QServoMessage>>,
    qt_gl: Option<(Device, Context)>,
}

impl Drop for QServoRendererRust {
    fn drop(&mut self) {
        self.sender
            .as_ref()
            .unwrap()
            .send(QServoMessage::Quit)
            .unwrap();

        if let Some((device, mut context)) = self.qt_gl.take() {
            device.destroy_context(&mut context).unwrap();
        }
    }
}

impl qobject::QServoRenderer {
    pub(crate) fn new() -> cxx::UniquePtr<Self> {
        qobject::qservo_renderer_unique_ptr()
    }

    fn render(mut self: Pin<&mut Self>) {
        // Ask to borrow a surface
        let (take_sender, take_receiver) = mpsc::sync_channel(0);
        let (recycle_sender, recycle_receiver) = mpsc::sync_channel(0);
        self.sender
            .as_ref()
            .unwrap()
            .send(QServoMessage::BorrowSurface(take_sender, recycle_receiver))
            .unwrap();

        // Wait for the response from the background thread
        let surface = take_receiver.recv();
        if let Ok(Some(surface)) = surface {
            // Find the target fbo
            let fbo_target = self.as_ref().framebuffer_object();

            if let Some((ref mut device, ref mut context)) = self.as_mut().rust_mut().qt_gl.as_mut()
            {
                // Build a texture from the surface
                match device.create_surface_texture(context, surface) {
                    Ok(texture) => {
                        // Retrieve the texture info
                        let object = device.surface_texture_object(&texture) as u32;
                        let target = device.surface_gl_texture_target() as u32;

                        // Build a source FBO from the texture
                        let fbo_source =
                            qobject::fbo_from_texture(object, target, QSize::new(400, 400));

                        println!("render!");

                        // Blit source FBO to the target FBO
                        unsafe { qobject::blit_framebuffer(fbo_target, fbo_source) };

                        // Destory the texture and return the surface back to the background thread
                        let surface = device.destroy_surface_texture(context, texture);
                        recycle_sender.send(surface.ok()).unwrap();
                    }
                    Err((_, surface)) => {
                        // Return the surface back to the background thread
                        recycle_sender.send(Some(surface)).unwrap();
                    }
                }
            } else {
                recycle_sender.send(Some(surface)).unwrap();
            }
        } else {
            recycle_sender.send(None).unwrap();
        }
    }

    unsafe fn synchronize(mut self: Pin<&mut Self>, item: *mut qobject::QQuickFramebufferObject) {
        println!("sync start");
        let webview_ptr = item as *mut ServoWebView;
        if let Some(webview_ref) = webview_ptr.as_mut() {
            let mut webview = Pin::new_unchecked(webview_ref);

            // Start the Servo worker thread if there isn't one
            if self.as_ref().sender.is_none() {
                let qt_thread = webview.qt_thread();
                let (sender, receiver) = mpsc::channel();

                use surfman::platform::generic::multi;
                use surfman::platform::unix::wayland;
                let native_connection = wayland::connection::NativeConnection::current()
                    .expect("Failed to bootstrap native connection");
                let wayland_connection = unsafe {
                    wayland::connection::Connection::from_native_connection(native_connection)
                        .expect("Failed to bootstrap wayland connection")
                };
                let connection = multi::connection::Connection::Default(
                    multi::connection::Connection::Default(wayland_connection),
                );

                std::thread::spawn(move || {
                    QServoThread::new(receiver, qt_thread, connection).run()
                });

                self.as_mut().rust_mut().sender = Some(sender);
            }

            // Check if we have a new URL
            let url = webview.as_ref().url().clone();
            if url != self.url {
                self.as_mut().rust_mut().url = url;

                let servo_url = ServoUrl::from_url(url::Url::try_from(&self.url).unwrap());
                self.as_ref()
                    .sender
                    .as_ref()
                    .unwrap()
                    .send(QServoMessage::Url(servo_url))
                    .unwrap();

                // Clear any favicon
                webview.as_mut().set_favicon_url(QUrl::default());
            }

            // Process any pending events
            self.as_ref()
                .sender
                .as_ref()
                .unwrap()
                .send(QServoMessage::Heartbeat)
                .unwrap();
        }

        println!("sync end");
    }

    // unsafe fn synchronize(mut self: Pin<&mut Self>, item: *mut qobject::QQuickFramebufferObject) {
    //     println!("sync start");
    //     let webview_ptr = item as *mut ServoWebView;
    //     if let Some(webview_ref) = webview_ptr.as_mut() {
    //         let mut webview = Pin::new_unchecked(webview_ref);

    //         // Ensure servo has been started
    //         if self.as_ref().servo.is_none() {
    //             if !self.as_mut().init_servo(webview.as_mut()) {
    //                 // TODO: hack to wait for a window
    //                 return;
    //             }

    //             println!("created servo");
    //         }

    //         // Check if we have a new URL
    //         let url = webview.as_ref().url().clone();
    //         if url != self.url {
    //             self.as_mut().rust_mut().url = url;

    //             let servo_url = ServoUrl::from_url(url::Url::try_from(&self.url).unwrap());

    //             // Open a new browser or load the url
    //             if let Some(browser_id) = self.browser.browser_id() {
    //                 self.as_mut()
    //                     .rust_mut()
    //                     .browser
    //                     .push_event(EmbedderEvent::LoadUrl(browser_id, servo_url));
    //             } else {
    //                 let browser_id = self.as_ref().browser_id.unwrap();
    //                 self.as_mut()
    //                     .rust_mut()
    //                     .browser
    //                     .push_event(EmbedderEvent::NewBrowser(servo_url, browser_id));
    //             }

    //             // Clear any favicon
    //             webview.as_mut().set_favicon_url(QUrl::default());
    //         }

    //         // Process any pending events
    //         self.as_mut().handle_events(webview);
    //     }

    //     println!("sync end");
    // }

    // fn init_servo(mut self: Pin<&mut Self>, mut webview: Pin<&mut ServoWebView>) -> bool {
    //     let event_loop_waker = QServoEventsLoopWaker::new(webview.as_mut().qt_thread());
    //     let embedder = Box::new(QServoEmbedder::new(event_loop_waker.clone_box()));

    //     // TODO: have real width and height later
    //     let window = QServoWindowHeadless::new(Size2D::new(400, 400));
    //     // Try later until there is a connection
    //     if window.is_err() {
    //         println!("waiting for window");
    //         // TODO: hack to wait for a window
    //         event_loop_waker.wake();
    //         return false;
    //     }

    //     println!("found window");

    //     let window = Rc::new(window.unwrap());
    //     let user_agent = None;
    //     // The in-process interface to Servo.
    //     //
    //     // It does everything necessary to render the web, primarily
    //     // orchestrating the interaction between JavaScript, CSS layout,
    //     // rendering, and the client window.
    //     //
    //     // Clients create a `Servo` instance for a given reference-counted type
    //     // implementing `WindowMethods`, which is the bridge to whatever
    //     // application Servo is embedded in. Clients then create an event
    //     // loop to pump messages between the embedding application and
    //     // various browser components.
    //     let servo_data = Servo::new(embedder, window.clone(), user_agent);

    //     self.as_mut().rust_mut().browser_id = Some(servo_data.browser_id);

    //     // Enable logging and store servo instance
    //     servo_data.servo.setup_logging();
    //     self.as_mut().rust_mut().servo = Some(servo_data.servo);

    //     // Initialise servo
    //     event_loop_waker.wake();

    //     true
    // }

    // pub(crate) fn handle_events(mut self: Pin<&mut Self>, mut webview: Pin<&mut ServoWebView>) {
    //     println!("handle start");
    //     if self.servo.is_none() {
    //         return;
    //     }

    //     // Browser process servo events
    //     let servo_events = self
    //         .as_mut()
    //         .rust_mut()
    //         .servo
    //         .as_mut()
    //         .unwrap()
    //         .get_events();
    //     let response = self
    //         .as_mut()
    //         .rust_mut()
    //         .browser
    //         .handle_servo_events(servo_events);

    //     // Handle the responses from browser events to Qt
    //     if let Some(title) = response.title {
    //         webview.as_mut().set_title(QString::from(&title));
    //     }
    //     if let Some(loading) = response.loading {
    //         webview.as_mut().set_loading(loading);
    //     }
    //     if let Some(favicon_url) = response.favicon_url {
    //         webview.as_mut().set_favicon_url(QUrl::from(&favicon_url));
    //     }
    //     // if let Some(present) = response.present {
    //     //     self.as_mut().rust_mut().need_present = present;
    //     // }

    //     // Servo process browser events
    //     let browser_events = self.as_mut().rust_mut().browser.get_events();
    //     self.as_mut()
    //         .rust_mut()
    //         .servo
    //         .as_mut()
    //         .unwrap()
    //         .handle_events(browser_events);

    //     println!("handle end");
    // }
}

impl cxx_qt::Initialize for qobject::QServoRenderer {
    fn initialize(mut self: core::pin::Pin<&mut Self>) {
        use surfman::platform::generic::multi;
        use surfman::platform::unix::wayland;
        let native_connection = wayland::connection::NativeConnection::current()
            .expect("Failed to bootstrap native connection");
        let wayland_connection = unsafe {
            wayland::connection::Connection::from_native_connection(native_connection)
                .expect("Failed to bootstrap wayland connection")
        };
        let connection = multi::connection::Connection::Default(
            multi::connection::Connection::Default(wayland_connection),
        );
        let adapter = connection
            .create_software_adapter()
            .expect("Failed to create adapter");
        let device = connection
            .create_device(&adapter)
            .expect("Failed to bootstrap surfman device");
        let native_context = {
            let current = wayland::context::NativeContext::current()
                .expect("Failed to bootstrap native context");
            multi::context::NativeContext::Default(multi::context::NativeContext::Default(current))
        };
        let context = unsafe {
            device
                .create_context_from_native_context(native_context)
                .expect("Failed to bootstrap surfman context")
        };

        self.as_mut().rust_mut().qt_gl = Some((device, context));
    }
}
