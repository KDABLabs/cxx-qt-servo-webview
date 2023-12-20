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
        // #[qobject]
        // FIXME: Cannot use CXX-Qt features with no qobject macro
        // or cannot add multiple bases
        // #[base = "QQuickFramebufferObjectRendererWithQObject"]
        #[base = "QQuickFramebufferObject::Renderer"]
        // #[base = "QObject"]
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
    swapchain::QServoSwapChain,
    webviewfbo::qobject::ServoWebView,
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

#[derive(Default)]
pub struct QServoRendererRust {
    // browser: QServoBrowser,
    // browser_id: Option<BrowserId>,
    // // need_preset: bool,
    // servo: Option<Servo<QServoWindowHeadless>>,
    url: QUrl,
    sender: Option<Sender<QServoMessage>>,
    swap_chain: Option<QServoSwapChain>,
}

impl Drop for QServoRendererRust {
    fn drop(&mut self) {
        println!("dropping!");
    }
}

impl qobject::QServoRenderer {
    pub(crate) fn new() -> cxx::UniquePtr<Self> {
        qobject::qservo_renderer_unique_ptr()
    }

    fn render(mut self: Pin<&mut Self>) {
        if self.as_ref().swap_chain.is_none() {
            let (sender, receiver) = mpsc::channel();
            self.sender
                .as_ref()
                .unwrap()
                .send(QServoMessage::GetSwapChain(sender))
                .unwrap();
            let data = receiver.recv_timeout(Duration::from_millis(16));
            if let Ok(data) = data {
                self.as_mut().rust_mut().swap_chain =
                    Some(QServoSwapChain::new(data.swap_chain, data.connection));
            } else {
                println!("no swap chain yet");
                return;
            }
        }

        println!("going to render!");

        // Build a source fbo
        if let Some(mut swap_chain) = self.as_mut().rust_mut().swap_chain.take() {
            {
                let (take_sender, take_receiver) = mpsc::channel();
                let (recycle_sender, recycle_receiver) = mpsc::channel();
                self.sender
                    .as_ref()
                    .unwrap()
                    .send(QServoMessage::BorrowSurface(take_sender, recycle_receiver))
                    .unwrap();

                let surface = take_receiver.recv();
                if let Ok(surface) = surface {
                    println!("render surface? {}", surface.is_some());

                    swap_chain.device.make_context_current(&swap_chain.context).unwrap();
                    println!("make context current");

                    let texture = surface.map(|surface| {
                        swap_chain
                            .device
                            .create_surface_texture(&mut swap_chain.context, surface)
                            .unwrap()
                    });
                    println!("texture: {}", texture.is_some());

                    if let Some(texture) = texture {
                        let surface = swap_chain
                            .device
                            .destroy_surface_texture(&mut swap_chain.context, texture);
                        recycle_sender.send(surface.ok()).unwrap();
                    } else {
                        recycle_sender.send(None).unwrap();
                    }
                }

                // let mut texture = swap_chain.take_surface_as_texture();

                // if texture.has_texture() {
                //     let fbo_source =
                //         qobject::fbo_from_texture(texture.object(), texture.target(), QSize::new(400, 400));
                //     // Find the target fbo
                //     let fbo_target = self.as_ref().framebuffer_object();

                //     println!("render!");

                //     // Blit it
                //     unsafe { qobject::blit_framebuffer(fbo_target, fbo_source) };
                // } else {
                //     println!("no texture, continue");
                // }
            }

            self.as_mut().rust_mut().swap_chain = Some(swap_chain);
        }
    }

    unsafe fn synchronize(mut self: Pin<&mut Self>, item: *mut qobject::QQuickFramebufferObject) {
        println!("sync start");
        let webview_ptr = item as *mut ServoWebView;
        if let Some(webview_ref) = webview_ptr.as_mut() {
            let mut webview = Pin::new_unchecked(webview_ref);

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

                std::thread::spawn(move || QServoThread::new(receiver, qt_thread, Some(connection)).run());

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


    // fn render(mut self: Pin<&mut Self>) {
        // println!("going to recomposite");

        // Ask for a frame to be rendered
        // self.as_mut()
        //     .rust_mut()
        //     .servo
        //     .as_mut()
        //     .unwrap()
        //     .recomposite();

        // {
        //     // Build a source fbo
        //     let swap_chain =
        //         QServoSwapChain::new(self.servo.as_ref().unwrap().window().webrender_surfman());
        //     let texture = swap_chain.take_surface_as_texture();

        //     if texture.has_texture() {
        //         let fbo_source = qobject::fbo_from_texture(
        //             texture.object(),
        //             texture.target(),
        //             QSize::new(400, 400),
        //         );
        //         // Find the target fbo
        //         let fbo_target = self.framebuffer_object();

        //         println!("render!");

        //         // Blit it
        //         unsafe { qobject::blit_framebuffer(fbo_target, fbo_source) };
        //     } else {
        //         println!("no texture, continue");
        //     }
        // }

        // println!("going to present");

        // Clear and onto the next frame
        // self.as_mut().rust_mut().servo.as_mut().unwrap().present();
    // }

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
    fn initialize(self: core::pin::Pin<&mut Self>) {}
}
