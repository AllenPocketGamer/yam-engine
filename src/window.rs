pub struct Window {
    pub(crate) window: winit::window::Window,
}

impl Window {
    pub(crate) fn new(window: winit::window::Window) -> Self {
        Self { window }
    }
}
