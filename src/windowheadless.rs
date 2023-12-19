use std::cell::Cell;

use servo::{
    compositing::windowing::{AnimationState, EmbedderCoordinates, WindowMethods},
    euclid::{Point2D, Scale, Size2D, Rect},
    webrender_surfman::WebrenderSurfman, servo_geometry::DeviceIndependentPixel,
};
use surfman::{Connection, SurfaceType, Error as SurfmanError};

pub(crate) struct QServoWindowHeadless {
    animation_state: Cell<AnimationState>,
    webrender_surfman: WebrenderSurfman,
}

impl QServoWindowHeadless {
    pub fn new(
        size: Size2D<u32, DeviceIndependentPixel>,
    ) -> Result<Self, SurfmanError> {
        use surfman::platform::generic::multi;
        use surfman::platform::unix::wayland;
        let native_connection = wayland::connection::NativeConnection::current()?;
        let wayland_connection = unsafe {
            wayland::connection::Connection::from_native_connection(native_connection)
                .expect("Failed to bootstrap wayland connection")
        };
        let connection = multi::connection::Connection::Default(
            multi::connection::Connection::Default(wayland_connection),
        );


        // Initialize surfman
        // let connection = Connection::new().expect("Failed to create connection");
        let adapter = connection
            .create_software_adapter()
            .expect("Failed to create adapter");
        let size = size.to_untyped().to_i32();
        let surface_type = SurfaceType::Generic { size };
        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        Ok(Self {
            webrender_surfman,
            animation_state: Cell::new(AnimationState::Idle),
        })
    }
}

impl WindowMethods for QServoWindowHeadless {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let size = self
            .webrender_surfman
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| Size2D::from_untyped(info.size))
            .unwrap_or(Size2D::new(0, 0));
        let origin = Point2D::origin();
        EmbedderCoordinates {
            hidpi_factor: Scale::new(1.0),
            screen: size,
            screen_avail: size,
            window: (size, origin),
            framebuffer: size,
            viewport: Rect::new(origin, size),
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn get_gl_context(&self) -> servo_media::player::context::GlContext {
        servo_media::player::context::GlContext::Unknown
    }

    fn get_native_display(&self) -> servo_media::player::context::NativeDisplay {
        servo_media::player::context::NativeDisplay::Unknown
    }

    fn get_gl_api(&self) -> servo_media::player::context::GlApi {
        servo_media::player::context::GlApi::OpenGL3
    }

    fn webrender_surfman(&self) -> WebrenderSurfman {
        self.webrender_surfman.clone()
    }
}
