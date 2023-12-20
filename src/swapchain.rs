use servo::webrender_surfman::WebrenderSurfman;

use surfman::chains::SwapChainAPI;
// use surfman::platform::generic::multi::device::Device;
use surfman::platform::generic::multi::surface::Surface;
use surfman::SurfaceTexture;
// use surfman::platform::unix::wayland::surface::Surface;

use surfman::platform::unix::generic::device::Device as GenericDevice;
use surfman::platform::unix::wayland::device::Device as WaylandDevice;
use surfman::platform::unix::x11::device::Device as X11Device;

use surfman::GLApi;

use surfman::{chains::SwapChain, Connection, Context, Device};

pub(crate) struct QServoTexture<'a> {
    texture: Option<SurfaceTexture>,
    swap_chain: &'a mut QServoSwapChain,
}

impl<'a> QServoTexture<'a> {
    fn new(swap_chain: &'a mut QServoSwapChain) -> Self {
        let surface = swap_chain.swap_chain.take_surface();
        println!("surface: {}", surface.is_some());

        let texture = surface.map(|surface| {
            swap_chain
                .device
                .create_surface_texture(&mut swap_chain.context, surface)
                .unwrap()
        });
        println!("texture: {}", texture.is_some());

        Self {
            texture,
            swap_chain,
        }
    }

    pub(crate) fn has_texture(&self) -> bool {
        self.texture.is_some()
    }

    pub(crate) fn object(&self) -> u32 {
        if let Some(texture) = self.texture.as_ref() {
            self.swap_chain.device.surface_texture_object(texture) as u32
        } else {
            unimplemented!()
        }
    }

    pub(crate) fn target(&self) -> u32 {
        self.swap_chain.device.surface_gl_texture_target() as u32
    }
}

impl<'a> Drop for QServoTexture<'a> {
    fn drop(&mut self) {
        println!("dropping surface!");
        if let Some(texture) = self.texture.take() {
            let surface = self
                .swap_chain
                .device
                .destroy_surface_texture(&mut self.swap_chain.context, texture)
                .unwrap();
            self.swap_chain.swap_chain.recycle_surface(surface);
            println!("recycled!");
        }
    }
}

// pub(crate) struct QServoSurface {
//     surface: Option<Surface<Device<WaylandDevice, X11Device>, GenericDevice>>,
//     surfman: WebrenderSurfman,
// }

// impl QServoSurface {
//     pub(crate) fn as_texture(&self) -> QServoTexture {
//         QServoTexture {
//             surfman: self.surfman.clone(),
//             texture: self
//                 .surface
//                 .as_ref()
//                 .map(|surface| self.surfman.create_surface_texture(surface).unwrap()),
//         }
//     }
// }

pub(crate) struct QServoSwapChain {
    pub(crate) swap_chain: SwapChain<Device>,
    pub(crate) connection: Connection,
    pub(crate) device: Device,
    pub(crate) context: Context,
}

impl Drop for QServoSwapChain {
    fn drop(&mut self) {
        self.device.destroy_context(&mut self.context).unwrap();
    }
}

impl QServoSwapChain {
    pub(crate) fn new(swap_chain: SwapChain<Device>, connection: Connection) -> Self {
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
        let adapter = connection
            .create_software_adapter()
            .expect("Failed to create adapter");
        let device = connection
            .create_device(&adapter)
            .expect("Failed to bootstrap surfman device");
        let native_context = {
            use surfman::platform::generic::multi;
            use surfman::platform::unix::wayland;
            multi::context::NativeContext::Default(multi::context::NativeContext::Default(
                wayland::context::NativeContext::current()
                    .expect("Failed to bootstrap native context"),
            ))
        };
        let context = unsafe {
            device
                .create_context_from_native_context(native_context)
                .expect("Failed to bootstrap surfman context")
        };

        Self {
            swap_chain,
            connection,
            device,
            context,
        }
    }

    // pub(crate) fn recycle_surface(&self, surface: Box<QServoSurface>) {
    //     let swap_chain = self.surfman.swap_chain().unwrap();
    //     if let Some(surface) = surface.surface {
    //         swap_chain.recycle_surface(surface)
    //     }
    // }

    pub(crate) fn take_surface_as_texture(&mut self) -> QServoTexture {
        // let webrender_gl = match self.surfman.connection().gl_api() {
        //     GLApi::GL => unsafe {
        //         gleam::gl::GlFns::load_with(|s| self.surfman.get_proc_address(s))
        //     },
        //     GLApi::GLES => unsafe {
        //         gleam::gl::GlesFns::load_with(|s| self.surfman.get_proc_address(s))
        //     },
        // };
        // self.surfman.make_gl_context_current().unwrap();
        // debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR);
        QServoTexture::new(self)
    }
}

// use servo::webrender_surfman::WebrenderSurfman;

// use surfman::{
//     chains::{SwapChain, SwapChainAPI},
//     Device,
// };
// // use surfman::platform::generic::multi::device::Device;
// // use surfman::platform::generic::multi::surface::Surface;
// use surfman::SurfaceTexture;
// // use surfman::platform::unix::wayland::surface::Surface;

// use surfman::platform::unix::generic::device::Device as GenericDevice;
// use surfman::platform::unix::wayland::device::Device as WaylandDevice;
// use surfman::platform::unix::x11::device::Device as X11Device;

// use surfman::GLApi;

// use surfman::{Connection, Context};

// pub(crate) struct QServoTexture {
//     texture: Option<SurfaceTexture>,
//     swap_chain: SwapChain<Device>,
//     device: Device,
//     context: Context,
// }

// impl QServoTexture {
//     // fn new(surfman: WebrenderSurfman) -> Self {
//     //     let swap_chain = surfman.swap_chain().unwrap();
//     //     let surface = swap_chain.take_surface();
//     //     println!("surface: {}", surface.is_some());
//     //     let texture = surface.map(|surface| surfman.create_surface_texture(surface).unwrap());
//     //     println!("texture: {}", texture.is_some());

//     //     Self { texture, surfman }
//     // }

//     fn new_from_swap(swap_chain: SwapChain<Device>) -> Self {
//         let surface = swap_chain.take_surface();
//         println!("surface: {}", surface.is_some());

//         let connection = Connection::new().expect("Failed to create connection");
//         let adapter = connection
//             .create_software_adapter()
//             .expect("Failed to create adapter");
//         let device = connection
//             .create_device(&adapter)
//             .expect("Failed to bootstrap surfman device");
//         let native_context = {
//             use surfman::platform::generic::multi;
//             use surfman::platform::unix::wayland;
//             multi::context::NativeContext::Default(multi::context::NativeContext::Default(
//                 wayland::context::NativeContext::current()
//                     .expect("Failed to bootstrap native context"),
//             ))
//         };
//         let mut context = unsafe {
//             device
//                 .create_context_from_native_context(native_context)
//                 .expect("Failed to bootstrap surfman context")
//         };

//         device.make_context_current(&context).unwrap();

//         let texture = surface.map(|surface| device.create_surface_texture(&mut context, surface).unwrap());
//         println!("texture: {}", texture.is_some());

//         Self { texture, swap_chain, device, context }
//     }

//     pub(crate) fn has_texture(&self) -> bool {
//         self.texture.is_some()
//     }

//     pub(crate) fn object(&self) -> u32 {
//         if let Some(texture) = self.texture.as_ref() {
//             self.device.surface_texture_object(texture) as u32
//         } else {
//             unimplemented!()
//         }
//     }

//     pub(crate) fn target(&self) -> u32 {
//         self.device.surface_gl_texture_target() as u32
//     }
// }

// impl Drop for QServoTexture {
//     fn drop(&mut self) {
//         println!("dropping surface!");
//         if let Some(texture) = self.texture.take() {
//             let surface = self.surfman.destroy_surface_texture(texture).unwrap();
//             let swap_chain = self.surfman.swap_chain().unwrap();
//             swap_chain.recycle_surface(surface);
//             println!("recycled!");
//         }
//     }
// }

// // pub(crate) struct QServoSurface {
// //     surface: Option<Surface<Device<WaylandDevice, X11Device>, GenericDevice>>,
// //     surfman: WebrenderSurfman,
// // }

// // impl QServoSurface {
// //     pub(crate) fn as_texture(&self) -> QServoTexture {
// //         QServoTexture {
// //             surfman: self.surfman.clone(),
// //             texture: self
// //                 .surface
// //                 .as_ref()
// //                 .map(|surface| self.surfman.create_surface_texture(surface).unwrap()),
// //         }
// //     }
// // }

// pub(crate) struct QServoSwapChain {
//     surfman: WebrenderSurfman,
// }

// impl QServoSwapChain {
//     pub(crate) fn new(surfman: WebrenderSurfman) -> Self {
//         Self { surfman }
//     }

//     // pub(crate) fn recycle_surface(&self, surface: Box<QServoSurface>) {
//     //     let swap_chain = self.surfman.swap_chain().unwrap();
//     //     if let Some(surface) = surface.surface {
//     //         swap_chain.recycle_surface(surface)
//     //     }
//     // }

//     pub(crate) fn take_surface_as_texture(&self) -> Box<QServoTexture> {
//         // let webrender_gl = match self.surfman.connection().gl_api() {
//         //     GLApi::GL => unsafe {
//         //         gleam::gl::GlFns::load_with(|s| self.surfman.get_proc_address(s))
//         //     },
//         //     GLApi::GLES => unsafe {
//         //         gleam::gl::GlesFns::load_with(|s| self.surfman.get_proc_address(s))
//         //     },
//         // };
//         // self.surfman.make_gl_context_current().unwrap();
//         // debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR);

//         Box::new(QServoTexture::new(self.surfman.clone()))
//     }
// }
