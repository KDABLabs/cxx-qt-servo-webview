#[cxx_qt::bridge(cxx_file_stem = "servowebview")]
pub(crate) mod qobject {
    // use servo::Servo;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;

        include!("qservorendernode.h");
    }

    extern "Rust" {
        type QServoTexture;
        type QServoSwapChain;

        fn take_surface_as_texture(self: &QServoSwapChain) -> Box<QServoTexture>;

        // fn recycle_surface(self: &QServoSwapChain, surface: Box<QServoSurface>);
        // fn take_surface(self: &QServoSwapChain) -> Box<QServoSurface>;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QServoRenderNode"]
        #[qml_element]
        #[qproperty(QUrl, favicon_url)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, title)]
        #[qproperty(QUrl, url)]
        type ServoWebView = super::QServoWebViewRust;

        #[qinvokable]
        fn swap_chain(self: &ServoWebView) -> Box<QServoSwapChain>;

        #[inherit]
        fn update(self: Pin<&mut ServoWebView>);
    }

    impl cxx_qt::Constructor<()> for ServoWebView {}
    impl cxx_qt::Threading for ServoWebView {}
}

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QUrl};
use servo::compositing::windowing::{EmbedderEvent, WindowMethods};
use servo::embedder_traits::EventLoopWaker;
use servo::euclid::Size2D;
use servo::servo_url::ServoUrl;
use servo::Servo;
use std::rc::Rc;
use surfman::chains::SwapChainAPI;
use surfman::GLApi;

use crate::browser::QServoBrowser;
use crate::embedder::QServoEmbedder;
use crate::events_loop::QServoEventsLoopWaker;
use crate::swapchain::{QServoTexture, QServoSwapChain};
use crate::window::QServoWindow;
use crate::windowheadless::QServoWindowHeadless;

#[derive(Default)]
pub struct QServoWebViewRust {
    browser: QServoBrowser,
    favicon_url: QUrl,
    loading: bool,
    servo: Option<Servo<QServoWindowHeadless>>,
    title: QString,
    // swap_chain: QServoSwapChain,
    url: QUrl,
}

impl QServoWebViewRust {
    fn as_servo_url(&self) -> Result<ServoUrl, url::ParseError> {
        Ok(ServoUrl::from_url(url::Url::try_from(&self.url)?))
    }
}

impl qobject::ServoWebView {
    pub fn swap_chain(&self) -> Box<QServoSwapChain> {
        Box::new(QServoSwapChain::new(
            self.servo.as_ref().unwrap().window().webrender_surfman(),
        ))
    }

    // pub fn render_swap_chain(&self) {
    //     let surfman = self
    //         .as_mut()
    //         .rust_mut()
    //         .servo
    //         .as_ref()
    //         .unwrap()
    //         .window()
    //         .webrender_surfman();
    //     let swap_chain = surfman.swap_chain().unwrap();
    //     let surface = swap_chain.take_surface();
    //     // TODO: render the surface to GL somewhere

    //     swap_chain.recycle_surface(surface);
    // }

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
            // Does a swap in surfman present
            // servo/components/webrender_surfman/lib.rs
            //
            // Note winit does this after the update() below
            // but we need this here as our update() happens "later",
            // so swap the buffers before compositing rather than after
            self.as_mut().rust_mut().servo.as_mut().unwrap().present();

            // Renders to the gl context ? via composite_specific_target
            // servo/components/compositing/compositor.rs
            self.as_mut()
                .rust_mut()
                .servo
                .as_mut()
                .unwrap()
                .recomposite();

            // let surfman = self
            //     .as_mut()
            //     .rust_mut()
            //     .servo
            //     .as_ref()
            //     .unwrap()
            //     .window()
            //     .webrender_surfman();
            // let swap_chain = surfman.swap_chain().unwrap();
            // if let Some(surface) = swap_chain.take_surface() {
            //     swap_chain.recycle_surface(surface);
            // }

            // TODO: get this surface to Qt
            // TODO: swap_chain.recycle_surface when it comes back

            // TODO: winit does a paint here
            // Schedule an updatePaintNode
            self.as_mut().update();
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

                // TODO: Should we create headless or not?
                // let window = Rc::new(QServoWindow::from_qwindow());
                let window = Rc::new(QServoWindowHeadless::new(Size2D::new(400, 400)));
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

                // Load the webrender
                // qobject.as_mut().rust_mut().swap_chain.set_webrender_surfman(window.webrender_surfman());

                // let webrender_surfman = window.webrender_surfman();
                // let webrender_gl = match webrender_surfman.connection().gl_api() {
                //     GLApi::GL => unsafe {
                //         gleam::gl::GlFns::load_with(|s| webrender_surfman.get_proc_address(s))
                //     },
                //     GLApi::GLES => unsafe {
                //         gleam::gl::GlesFns::load_with(|s| webrender_surfman.get_proc_address(s))
                //     },
                // };
                // webrender_surfman.make_gl_context_current().unwrap();
                // debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR);

                // let gl = unsafe {
                //     glow::Context::from_loader_function(|s| webrender_surfman.get_proc_address(s))
                // };

                // // Adapted from https://github.com/emilk/egui/blob/9478e50d012c5138551c38cbee16b07bc1fcf283/crates/egui_glow/examples/pure_glow.rs
                // let context = EguiGlow::new(events_loop.as_winit(), Arc::new(gl), None);
                // context
                //     .egui_ctx
                //     .set_pixels_per_point(window.hidpi_factor().get());

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
