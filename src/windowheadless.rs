// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::cell::Cell;

use servo::{
    compositing::windowing::{AnimationState, EmbedderCoordinates, WindowMethods},
    euclid::{Point2D, Rect, Scale, Size2D},
    rendering_context::RenderingContext,
    servo_geometry::DeviceIndependentPixel,
};
use surfman::{Connection, SurfaceType};

pub(crate) struct QServoWindowHeadless {
    animation_state: Cell<AnimationState>,
    rendering_context: RenderingContext,
}

impl QServoWindowHeadless {
    pub fn new(size: Size2D<u32, DeviceIndependentPixel>, connection: Connection) -> Self {
        // Initialize surfman
        let adapter = connection
            .create_software_adapter()
            .expect("Failed to create adapter");
        let size = size.to_untyped().to_i32();
        let surface_type = SurfaceType::Generic { size };
        let rendering_context = RenderingContext::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        Self {
            rendering_context,
            animation_state: Cell::new(AnimationState::Idle),
        }
    }
}

impl WindowMethods for QServoWindowHeadless {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let size = self
            .rendering_context
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

    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context.clone()
    }
}
