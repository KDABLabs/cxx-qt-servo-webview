// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashMap, rc::Rc};

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
use surfman::{chains::SwapChainAPI, SurfaceTexture};
use surfman::{Connection, Surface};
use url::Url;

use crate::{
    browser::QServoBrowser, embedder::QServoEmbedder, events_loop::QServoEventsLoopWaker,
    webview::qobject::ServoWebView, windowheadless::QServoWindowHeadless,
};

pub(crate) struct QServoHelper {
    browser: QServoBrowser,
    browser_id: TopLevelBrowsingContextId,
    servo: Servo<QServoWindowHeadless>,
    current_url: Option<ServoUrl>,
    favicons: HashMap<Url, Url>,
}

impl QServoHelper {
    pub(crate) fn new(qt_thread: CxxQtThread<ServoWebView>, connection: Connection) -> Self {
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
            current_url: None,
            favicons: HashMap::<Url, Url>::new(),
        }
    }

    pub(crate) fn navigation(&mut self, direction: i32) {
        let direction = if direction < 0 {
            servo::msg::constellation_msg::TraversalDirection::Back(
                direction.unsigned_abs() as usize
            )
        } else {
            servo::msg::constellation_msg::TraversalDirection::Forward(
                direction.unsigned_abs() as usize
            )
        };

        self.browser
            .push_event(EmbedderEvent::Navigation(self.browser_id, direction));
    }

    pub(crate) fn raw_embedded_event(&mut self, event: EmbedderEvent) {
        self.browser.push_event(event);
    }

    pub(crate) fn resize(&mut self, size: Size2D<i32, DevicePixel>) {
        let surfman = self.servo.window().rendering_context();
        surfman
            .resize(size.to_untyped().to_i32())
            .expect("Failed to resize");
        self.browser.push_event(EmbedderEvent::Resize);
    }

    pub(crate) fn url(&mut self, url: ServoUrl) {
        // Don't update the url if this was the last url
        if Some(&url) == self.current_url.as_ref() {
            return;
        }

        self.current_url = Some(url.clone());

        // Open a new browser or load the url
        if let Some(webview_id) = self.browser.webview_id() {
            self.browser
                .push_event(EmbedderEvent::LoadUrl(webview_id, url));
        } else {
            self.browser
                .push_event(EmbedderEvent::NewWebView(url, self.browser_id));
        }
    }

    pub(crate) fn borrow_surface_texture(&mut self) -> Option<(SurfaceTexture, u32, u32)> {
        let rendering_context = self.servo.window().rendering_context();
        let swap_chain = rendering_context.swap_chain().unwrap();

        let surface = swap_chain.take_surface();
        println!("sending surface: {}", surface.is_some());

        let mut result = None;
        if let Some(surface) = surface {
            result = match rendering_context.create_surface_texture(surface) {
                Ok(texture) => {
                    // Retrieve the texture info
                    let object = rendering_context.surface_texture_object(&texture);
                    let target = rendering_context.device().surface_gl_texture_target();

                    println!("created surface");

                    Some((texture, object, target))
                }
                Err((_, surface)) => {
                    println!("error creating surface");
                    swap_chain.recycle_surface(surface);
                    None
                }
            };
        }

        result
    }

    pub(crate) fn recycle_surface_texture(&mut self, texture: SurfaceTexture) {
        let rendering_context = self.servo.window().rendering_context();
        let swap_chain = rendering_context.swap_chain().unwrap();

        println!("returned surface, recycling");
        if let Ok(surface) = rendering_context.destroy_surface_texture(texture) {
            swap_chain.recycle_surface(surface);
        }
    }

    pub(crate) fn heartbeat(
        &mut self,
        navigation_allowed: bool,
        qt_thread: CxxQtThread<ServoWebView>,
    ) {
        // Browser process servo events
        let mut need_present = false;
        let mut need_resize = false;

        {
            let mut servo_events = self.servo.get_events();
            loop {
                let mut response = self
                    .browser
                    .handle_servo_events(servo_events, navigation_allowed);

                if let Some(url) = response.url.as_ref() {
                    // If there is no favicon but we have found one previously
                    // for this url then set it
                    if response.favicon_url.is_none() {
                        if let Some(favicon) = self.favicons.get(url) {
                            response.favicon_url = Some(favicon.to_owned());
                        }
                    }

                    self.current_url = Some(ServoUrl::from_url(url.to_owned()));
                }

                // Store the favicon for the current url
                if let Some(favicon) = response.favicon_url.as_ref() {
                    if let Some(current_url) = self.current_url.as_ref() {
                        self.favicons
                            .insert(current_url.as_url().to_owned(), favicon.to_owned());
                    }
                }

                // If there is a url but no known favicon then set an empty icon for now
                // so that when loading we don't show the old favicon
                if response.url.is_some() && response.favicon_url.is_none() {
                    response.favicon_url =
                        Some(Url::parse("https://localhost/emptyfavicon.ico").unwrap());
                }

                // Handle the responses from browser events to Qt
                qt_thread
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
                        if let Some(url) = response.blocked_navigation_request {
                            webview
                                .as_mut()
                                .blocked_navigation_request(QUrl::from(&url));
                        }
                        if let Some(can_go_back) = response.can_go_back {
                            webview.as_mut().set_can_go_back(can_go_back);
                        }
                        if let Some(can_go_forward) = response.can_go_forward {
                            webview.as_mut().set_can_go_forward(can_go_forward);
                        }
                    })
                    .unwrap();

                // Present when required
                need_present |= response.present.unwrap_or(false);

                // Servo process browser events
                let browser_events = self.browser.get_events();
                need_resize |= self.servo.handle_events(browser_events);

                servo_events = self.servo.get_events();
                // There could be more events so loop around if there are
                if servo_events.len() == 0 {
                    break;
                }
            }
        }

        // If we have resized then we need to force a repaint synchornously
        //
        // This is the same as Present::Immediate in servoshell
        // Resizes are unusual in that we need to repaint synchronously.
        // TODO(servo#30049) can we replace this with the simpler Servo::recomposite?
        if need_resize {
            println!("repaint_synchronously");
            self.servo.repaint_synchronously();
        }

        // Instead we do this after recycle_surface as this avoids issues
        // with the surface becoming empty after resize
        // Present if we resized or need to present
        if need_present || need_resize {
            println!("present!");
            self.servo.present();
        }

        // If we are resizing then recomposite after the present
        // as we need to ensure that the front and back buffer
        // have both been recomposite / repaint_synchronously
        if need_resize {
            println!("recomposite");
            self.servo.recomposite();
        }

        println!("heartbeat!");
    }

    pub(crate) fn quit(&mut self) {
        println!("quiting!");
        self.servo.handle_events(vec![EmbedderEvent::Quit]);
    }
}
