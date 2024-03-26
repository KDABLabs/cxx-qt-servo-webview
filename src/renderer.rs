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

        #[cxx_name = "resetOpenGLState"]
        fn reset_opengl_state();
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

use crate::{servohelper::QServoHelper, webview::qobject::ServoWebView};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QSize, QUrl};
use euclid::Size2D;
use servo::{compositing::windowing::EmbedderEvent, servo_url::ServoUrl};
use surfman::{Connection, Context, Device};

#[derive(Default)]
pub struct QServoRendererRust {
    size: QSize,
    url: QUrl,
    servo: Option<QServoHelper>,
    qt_gl: Option<(Device, Context)>,
}

impl Drop for QServoRendererRust {
    fn drop(&mut self) {
        if let Some(servo) = self.servo.as_mut() {
            servo.quit();
        }
    }
}

impl qobject::QServoRenderer {
    pub(crate) fn new() -> cxx::UniquePtr<Self> {
        qobject::qservo_renderer_unique_ptr()
    }

    fn render(mut self: Pin<&mut Self>) {
        println!("render start");

        // Find the target fbo
        let fbo_target = self.as_ref().framebuffer_object();
        let size = self.as_ref().size.clone();

        let mut rust_mut = self.as_mut().rust_mut();
        if let Some(servo) = rust_mut.servo.as_mut() {
            if let Some((texture, object, target)) = servo.borrow_surface_texture2() {
                // Build a source FBO from the texture
                //
                // Note that this is a unique_ptr which is freed when bliting
                let fbo_source = qobject::fbo_from_texture(object, target, size);

                // Blit source FBO to the target FBO
                unsafe { qobject::blit_framebuffer(fbo_target, fbo_source) };

                servo.recycle_surface_texture2(texture);
            }

            // Make Qt the context again
            //if let Some((ref mut device, ref mut context)) = servo.qt_gl.as_mut() {
            //    device.make_context_current(context).unwrap();
            //}
        }

        println!("render end");

        qobject::reset_opengl_state();

        println!("reset opengl state!");
    }

    unsafe fn synchronize(mut self: Pin<&mut Self>, item: *mut qobject::QQuickFramebufferObject) {
        println!("sync start");
        let webview_ptr = item as *mut ServoWebView;
        if let Some(webview_ref) = webview_ptr.as_mut() {
            let mut webview = Pin::new_unchecked(webview_ref);

            // Start the Servo worker thread if there isn't one
            if self.as_ref().servo.is_none() {
                let qt_thread = webview.qt_thread();

                let connection = Connection::new().expect("Failed to create connection");

                // use surfman::platform::generic::multi;
                // use surfman::platform::unix::wayland;
                // let native_connection = wayland::connection::NativeConnection::current()
                //     .expect("Failed to bootstrap native connection");
                // let wayland_connection = unsafe {
                //     wayland::connection::Connection::from_native_connection(native_connection)
                //         .expect("Failed to bootstrap wayland connection")
                // };
                // let connection = multi::connection::Connection::Default(
                //     multi::connection::Connection::Default(wayland_connection),
                // );

                let qt_gl = self.as_mut().rust_mut().qt_gl.take();
                self.as_mut().rust_mut().servo =
                    Some(QServoHelper::new(qt_thread, connection, qt_gl));
            }

            // Check if we have a new URL
            let url = webview.as_ref().url().clone();
            if url != self.url && url.is_valid() {
                self.as_mut().rust_mut().url = url;

                let new_url = url::Url::try_from(&self.url);
                if new_url.is_ok() {
                    if let Some(servo) = self.as_mut().rust_mut().servo.as_mut() {
                        servo.url(ServoUrl::from_url(new_url.unwrap()));
                    }
                }
            }

            let size = webview.as_ref().size().to_size();
            if size != self.size {
                self.as_mut().rust_mut().size = size;

                let (width, height) = (self.size.width(), self.size.height());
                if let Some(servo) = self.as_mut().rust_mut().servo.as_mut() {
                    servo.resize(Size2D::new(width, height));
                }
            }

            if let Some(direction) = webview.as_mut().rust_mut().navigation_direction.take() {
                if let Some(servo) = self.as_mut().rust_mut().servo.as_mut() {
                    servo.navigation(direction);
                }
            }

            // Process any converted events from Qt
            let events: Vec<EmbedderEvent> = webview.as_mut().rust_mut().events.drain(..).collect();
            for event in events.into_iter() {
                if let Some(servo) = self.as_mut().rust_mut().servo.as_mut() {
                    servo.raw_embedded_event(event);
                }
            }

            // Process any pending events
            let navigation_allowed = *webview.as_ref().navigation_allowed();
            let qt_thread = webview.qt_thread();
            if let Some(servo) = self.as_mut().rust_mut().servo.as_mut() {
                servo.heartbeat(navigation_allowed, qt_thread);
            }

            webview.as_mut().update();
        }

        println!("sync end");
    }
}

impl cxx_qt::Initialize for qobject::QServoRenderer {
    fn initialize(mut self: core::pin::Pin<&mut Self>) {
        use surfman::platform::generic::multi;
        use surfman::platform::unix::generic::context::NativeContext;
        let connection = Connection::new().unwrap();
        let adapter = connection
            .create_software_adapter()
            .expect("Failed to create adapter");
        let device = connection
            .create_device(&adapter)
            .expect("Failed to bootstrap surfman device");
        let native_context = NativeContext::current().unwrap();
        let context = unsafe {
            device
                .create_context_from_native_context(surfman::NativeContext::Default(
                    multi::context::NativeContext::Default(native_context),
                ))
                .expect("Failed to bootstrap surfman context")
        };

        self.as_mut().rust_mut().qt_gl = Some((device, context));
    }
}
