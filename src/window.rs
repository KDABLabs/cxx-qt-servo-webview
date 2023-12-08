use std::cell::Cell;

use euclid::{Rect, Size2D, Point2D, Scale};
use servo::compositing::windowing::{AnimationState, WindowMethods};
use servo::servo_config::pref;
use servo::webrender_surfman::WebrenderSurfman;
use servo_media::player::context::{GlApi, GlContext as PlayerGLContext, NativeDisplay};
#[cfg(target_os = "linux")]
use surfman::platform::generic::multi::connection::NativeConnection;
#[cfg(target_os = "linux")]
use surfman::platform::generic::multi::context::NativeContext;
use surfman::{Connection, GLApi, GLVersion, SurfaceType};

#[repr(transparent)]
struct c_void(std::ffi::c_void);

unsafe impl cxx::ExternType for c_void {
    type Id = cxx::type_id!("rust::servo::c_void");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    #[namespace = "rust::servo"]
    unsafe extern "C++" {
        include!("platformhelpers.h");
        type c_void = super::c_void;

        #[rust_name = "wayland_display_handle"]
        unsafe fn waylandDisplayHandle() -> *mut c_void;
        // #[rust_name = "x11_display_handle"]
        // unsafe fn x11DisplayHandle() -> *mut c_void;

        #[rust_name = "wayland_window_handle"]
        unsafe fn waylandWindowHandle() -> *mut c_void;
    }
}

pub(crate) struct QServoWindow {
    animation_state: Cell<AnimationState>,
    webrender_surfman: WebrenderSurfman,
}

impl QServoWindow {
    pub fn from_qwindow() -> Self {
        // Initialize surfman

        // https://github.com/slint-ui/slint/issues/877
        //
        // In C++ we can get the window handle for a QWidget/Window via QWindow::winId()
        // and the display connection via QPlatformNativeInterface, which supports a
        // per-window connection, for example for the xcb_connection_t.
        // After that we should be able to construct the different variants of
        // https://docs.rs/raw-window-handle/0.4.2/raw_window_handle/index.html .

        // TODO: convert QScreen::handle into raw display handle
        // https://docs.rs/raw-window-handle/0.5.2/raw_window_handle/enum.RawDisplayHandle.html
        // let display_handle = winit_window.raw_display_handle();
        // let display_handle = raw_window_handle::RawDisplayHandle::Wayland(
        //     raw_window_handle::WaylandDisplayHandle::empty(),
        // );
        // let ptr = unsafe { core::mem::transmute::<*mut ffi::c_void, *mut std::ffi::c_void>(ffi::display_handle()) };

        let display_handle = {
            let wayland_display_handle = unsafe { ffi::wayland_display_handle() };
            if !wayland_display_handle.is_null() {
                let mut wayland_handle = raw_window_handle::WaylandDisplayHandle::empty();
                wayland_handle.display = wayland_display_handle as *mut std::ffi::c_void;
                raw_window_handle::RawDisplayHandle::Wayland(wayland_handle)
            }
            else
            {
                unimplemented!();
                // let x11_display_handle = unsafe { ffi::x11_display_handle() };
                // if !x11_display_handle.is_null() {
                //     let mut x11_handle = raw_window_handle::XcbDisplayHandle::empty();
                //     // TODO: needs screen as well
                //     x11_handle.connection = x11_display_handle as *mut std::ffi::c_void;
                //     raw_window_handle::RawDisplayHandle::Xcb(x11_handle)
                // }
                // else
                // {
                //     panic!("Could not find display handle!");
                // }
            }
        };

        let connection = Connection::from_raw_display_handle(display_handle)
            .expect("Failed to create connection");
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");
        // TODO: convert QWindow::winid into raw window handle or a pointer
        //
        // The wl_surface* is already available through nativeResourceForWindow() in the QPA API
        //
        // https://blog.david-redondo.de/qt/kde/2022/12/09/wayland-native-interface.html
        //
        // https://docs.rs/surfman/0.8.1/surfman/connection/trait.Connection.html#tymethod.create_native_widget_from_ptr
        // https://docs.rs/surfman/0.8.1/surfman/connection/trait.Connection.html#tymethod.create_native_widget_from_rwh
        // let window_handle = winit_window.raw_window_handle();
        let window_handle = {
            let wayland_window_handle = unsafe { ffi::wayland_window_handle() };
            if !wayland_window_handle.is_null() {
                let mut wayland_handle = raw_window_handle::WaylandWindowHandle::empty();
                wayland_handle.surface = wayland_window_handle as *mut std::ffi::c_void;
                raw_window_handle::RawWindowHandle::Wayland(wayland_handle)
            }
            else
            {
                unimplemented!();
            }
        };
        let native_widget = connection
            .create_native_widget_from_rwh(window_handle)
            .expect("Failed to create native widget");
        let surface_type = SurfaceType::Widget { native_widget };
        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        Self {
            animation_state: Cell::new(AnimationState::Idle),
            webrender_surfman,
        }
    }
}

// https://doc.servo.org/compositing/windowing/trait.WindowMethods.html
impl WindowMethods for QServoWindow {
    fn get_coordinates(&self) -> servo::compositing::windowing::EmbedderCoordinates {
        // /// The pixel density of the display.
        // pub hidpi_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
        // /// Size of the screen.
        // pub screen: DeviceIntSize,
        // /// Size of the available screen space (screen without toolbars and docks).
        // pub screen_avail: DeviceIntSize,
        // /// Size of the native window.
        // pub window: (DeviceIntSize, DeviceIntPoint),
        // /// Size of the GL buffer in the window.
        // pub framebuffer: DeviceIntSize,
        // /// Coordinates of the document within the framebuffer.
        // pub viewport: DeviceIntRect,

        servo::compositing::windowing::EmbedderCoordinates {
            viewport: Rect::new(Point2D::new(0, 0), Size2D::new(400, 400)),
            framebuffer: Size2D::new(400, 400),
            window: (Size2D::new(400, 400), Point2D::new(0, 0)),
            screen: Size2D::new(400, 400),
            screen_avail: Size2D::new(400, 400),
            hidpi_factor: Scale::new(1.0),
        }
    }

    fn set_animation_state(&self, state: servo::compositing::windowing::AnimationState) {
        self.animation_state.set(state);
    }

    fn get_gl_context(&self) -> servo_media::player::context::GlContext {
        if !pref!(media.glvideo.enabled) {
            return PlayerGLContext::Unknown;
        }

        #[allow(unused_variables)]
        let native_context = self.webrender_surfman.native_context();

        #[cfg(target_os = "windows")]
        return PlayerGLContext::Egl(native_context.egl_context as usize);

        #[cfg(target_os = "linux")]
        return match native_context {
            NativeContext::Default(NativeContext::Default(native_context)) => {
                PlayerGLContext::Egl(native_context.egl_context as usize)
            }
            NativeContext::Default(NativeContext::Alternate(native_context)) => {
                PlayerGLContext::Egl(native_context.egl_context as usize)
            }
            NativeContext::Alternate(_) => unimplemented!(),
        };

        // @TODO(victor): https://github.com/servo/media/pull/315
        #[cfg(target_os = "macos")]
        #[allow(unreachable_code)]
        return unimplemented!();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return unimplemented!();
    }

    fn get_native_display(&self) -> servo_media::player::context::NativeDisplay {
        if !pref!(media.glvideo.enabled) {
            return NativeDisplay::Unknown;
        }

        #[allow(unused_variables)]
        let native_connection = self.webrender_surfman.connection().native_connection();
        #[allow(unused_variables)]
        let native_device = self.webrender_surfman.native_device();

        #[cfg(target_os = "windows")]
        return NativeDisplay::Egl(native_device.egl_display as usize);

        #[cfg(target_os = "linux")]
        return match native_connection {
            NativeConnection::Default(NativeConnection::Default(conn)) => {
                NativeDisplay::Egl(conn.0 as usize)
            }
            NativeConnection::Default(NativeConnection::Alternate(conn)) => {
                NativeDisplay::X11(conn.x11_display as usize)
            }
            NativeConnection::Alternate(_) => unimplemented!(),
        };

        // @TODO(victor): https://github.com/servo/media/pull/315
        #[cfg(target_os = "macos")]
        #[allow(unreachable_code)]
        return unimplemented!();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return unimplemented!();
    }

    fn get_gl_api(&self) -> GlApi {
        let api = self.webrender_surfman.connection().gl_api();
        let attributes = self.webrender_surfman.context_attributes();
        let GLVersion { major, minor } = attributes.version;
        match api {
            GLApi::GL if major >= 3 && minor >= 2 => GlApi::OpenGL3,
            GLApi::GL => GlApi::OpenGL,
            GLApi::GLES if major > 1 => GlApi::Gles2,
            GLApi::GLES => GlApi::Gles1,
        }
    }

    fn webrender_surfman(&self) -> servo::webrender_surfman::WebrenderSurfman {
        self.webrender_surfman.clone()
    }
}
