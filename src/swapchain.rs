use servo::webrender_surfman::WebrenderSurfman;

use surfman::chains::SwapChainAPI;
use surfman::platform::generic::multi::device::Device;
use surfman::platform::generic::multi::surface::Surface;
use surfman::SurfaceTexture;
// use surfman::platform::unix::wayland::surface::Surface;

use surfman::platform::unix::generic::device::Device as GenericDevice;
use surfman::platform::unix::wayland::device::Device as WaylandDevice;
use surfman::platform::unix::x11::device::Device as X11Device;

use surfman::GLApi;

pub(crate) struct QServoTexture {
    texture: Option<SurfaceTexture>,
    surfman: WebrenderSurfman,
}

impl QServoTexture {
    fn new(surfman: WebrenderSurfman) -> Self {
        let swap_chain = surfman.swap_chain().unwrap();
        let surface = swap_chain.take_surface();
        println!("surface: {}", surface.is_some());
        let texture = surface.map(|surface| surfman.create_surface_texture(surface).unwrap());
        println!("texture: {}", texture.is_some());

        Self { texture, surfman }
    }

    pub(crate) fn object(&self) -> u32 {
        if let Some(texture) = self.texture.as_ref() {
            self.surfman.surface_texture_object(texture) as u32
        } else {
            unimplemented!()
        }
    }

    pub(crate) fn target(&self) -> u32 {
        self.surfman.device().surface_gl_texture_target() as u32
    }
}

impl Drop for QServoTexture {
    fn drop(&mut self) {
        println!("dropping surface!");
        if let Some(texture) = self.texture.take() {
            let surface = self.surfman.destroy_surface_texture(texture).unwrap();
            let swap_chain = self.surfman.swap_chain().unwrap();
            swap_chain.recycle_surface(surface);
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
    surfman: WebrenderSurfman,
}

impl QServoSwapChain {
    pub(crate) fn new(surfman: WebrenderSurfman) -> Self {
        Self { surfman }
    }

    // pub(crate) fn recycle_surface(&self, surface: Box<QServoSurface>) {
    //     let swap_chain = self.surfman.swap_chain().unwrap();
    //     if let Some(surface) = surface.surface {
    //         swap_chain.recycle_surface(surface)
    //     }
    // }

    pub(crate) fn take_surface_as_texture(&self) -> Box<QServoTexture> {
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

        Box::new(QServoTexture::new(self.surfman.clone()))
    }
}
