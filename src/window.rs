pub struct Window {
    pub(crate) window: winit::window::Window,
}

impl Window {
    pub(crate) fn new(window: winit::window::Window) -> Self {
        Self { window }
    }

    pub fn inner_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();

        (size.width, size.height)
    }
}