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
            source: UniquePtr<QOpenGLFramebufferObject>,
        );

        include!("cxx-qt-lib/qsize.h");
        type QSize = cxx_qt_lib::QSize;

        #[cxx_name = "fboFromTexture"]
        fn fbo_from_texture(
            texture_id: u32,
            texture_target: u32,
            size: QSize,
        ) -> UniquePtr<QOpenGLFramebufferObject>;
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
    servothread::{QServoMessage, QServoThread},
    webview::qobject::ServoWebView,
};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QSize, QUrl};
use euclid::Size2D;
use servo::{compositing::windowing::EmbedderEvent, servo_url::ServoUrl};
use std::sync::mpsc::{self, Sender};
use surfman::{Context, Device};

#[derive(Default)]
pub struct QServoRendererRust {
    size: QSize,
    url: QUrl,
    servo_sender: Option<Sender<QServoMessage>>,
    qt_gl: Option<(Device, Context)>,
}

impl Drop for QServoRendererRust {
    fn drop(&mut self) {
        self.servo_sender
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
        println!("render start");

        // Ask to borrow a surface
        let (take_sender, take_receiver) = mpsc::sync_channel(0);
        let (recycle_sender, recycle_receiver) = mpsc::sync_channel(0);
        self.servo_sender
            .as_ref()
            .unwrap()
            .send(QServoMessage::BorrowSurface(take_sender, recycle_receiver))
            .unwrap();

        // Wait for the response from the background thread
        let surface = take_receiver.recv();
        if let Ok(Some(surface)) = surface {
            // Find the target fbo
            let fbo_target = self.as_ref().framebuffer_object();
            let size = self.as_ref().size.clone();

            if let Some((ref mut device, ref mut context)) = self.as_mut().rust_mut().qt_gl.as_mut()
            {
                // Build a texture from the surface
                match device.create_surface_texture(context, surface) {
                    Ok(texture) => {
                        // Retrieve the texture info
                        let object = device.surface_texture_object(&texture);
                        let target = device.surface_gl_texture_target();

                        // Build a source FBO from the texture
                        //
                        // Note that this is a unique_ptr which is freed when bliting
                        let fbo_source = qobject::fbo_from_texture(object, target, size);

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

        println!("render end");
    }

    unsafe fn synchronize(mut self: Pin<&mut Self>, item: *mut qobject::QQuickFramebufferObject) {
        println!("sync start");
        let webview_ptr = item as *mut ServoWebView;
        if let Some(webview_ref) = webview_ptr.as_mut() {
            let mut webview = Pin::new_unchecked(webview_ref);

            // Start the Servo worker thread if there isn't one
            if self.as_ref().servo_sender.is_none() {
                let qt_thread = webview.qt_thread();
                let (servo_sender, servo_receiver) = mpsc::channel();

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
                    QServoThread::new(servo_receiver, qt_thread, connection).run()
                });

                self.as_mut().rust_mut().servo_sender = Some(servo_sender);
            }

            // Check if we have a new URL
            let url = webview.as_ref().url().clone();
            if url != self.url {
                if url.is_valid() {
                    self.as_mut().rust_mut().url = url;

                    let new_url = url::Url::try_from(&self.url);
                    if new_url.is_ok() {
                        let servo_url = ServoUrl::from_url(new_url.unwrap());
                        self.as_ref()
                            .servo_sender
                            .as_ref()
                            .unwrap()
                            .send(QServoMessage::Url(servo_url))
                            .unwrap();
                    }
                }
            }

            let size = webview.as_ref().size().to_size();
            if size != self.size {
                self.as_mut().rust_mut().size = size;

                self.as_ref()
                    .servo_sender
                    .as_ref()
                    .unwrap()
                    .send(QServoMessage::Resize(Size2D::new(
                        self.size.width(),
                        self.size.height(),
                    )))
                    .unwrap();
            }

            if let Some(direction) = webview.as_mut().rust_mut().navigation_direction.take() {
                self.as_ref()
                    .servo_sender
                    .as_ref()
                    .unwrap()
                    .send(QServoMessage::Navigation(direction))
                    .unwrap();
            }

            // Process any converted events from Qt
            let events: Vec<EmbedderEvent> = webview.as_mut().rust_mut().events.drain(..).collect();
            for event in events.into_iter() {
                self.as_ref()
                    .servo_sender
                    .as_ref()
                    .unwrap()
                    .send(QServoMessage::RawEmbeddedEvent(event))
                    .unwrap();
            }

            // Process any pending events
            let navigation_allowed = *webview.as_ref().navigation_allowed();
            let (heartbeat_sender, heartbeat_receiver) = mpsc::sync_channel(0);
            self.as_ref()
                .servo_sender
                .as_ref()
                .unwrap()
                .send(QServoMessage::Heartbeat(
                    heartbeat_sender,
                    navigation_allowed,
                ))
                .unwrap();
            // Wait for response, otherwise if we enter render() while the
            // heartbeat is running flickering can occur
            heartbeat_receiver.recv().unwrap();
        }

        println!("sync end");
    }
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
