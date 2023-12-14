#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type QServoSurface;
        type QServoSwapChain;

        fn recycle_surface(self: &QServoSwapChain, surface: QServoSurface);
        fn take_surface(self: &QServoSwapChain) -> QServoSurface;
    }
}

#[derive(Default)]
pub(crate) struct QServoSurface;


#[derive(Default)]
pub(crate) struct QServoSwapChain {
    surfman: Option<WebrenderSurfman>,
}

impl QServoSwapChain {
    pub(crate) fn recycle_surface(&self, _surface: QServoSurface) {
        // let swap_chain = self.surfman.swap_chain().unwrap();
        // swap_chain.recycle_surface(surface.into())
    }

    pub(crate) fn set_webrender_surfman(&mut self, surfman: WebrenderSurfman) {
        self.surfman = Some(surfman);
    }

    pub(crate) fn take_surface(&self) -> QServoSurface {
        // let swap_chain = self.surfman.swap_chain().unwrap();
        // swap_chain.take_surface().into()

        QServoSurface::default()
    }
}
