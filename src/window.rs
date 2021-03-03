//! NOTE: The `window` module is only in a usable state and will be gradually improved afterwards.

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::Window as Window_w,
};

pub type VideoMode = winit::monitor::VideoMode;
pub type MonitorHandle = winit::monitor::MonitorHandle;
pub type Fullscreen = winit::window::Fullscreen;
pub type NotSupportError = winit::error::NotSupportedError;
pub type ExternalError = winit::error::ExternalError;

/// `Window` is a simple wrapper for `winit::window::Window`, which cuts out many functions of
/// `winit::window::Window`, only keeps what `yam` cares about.
pub struct Window {
    pub(crate) window: Window_w,
}

impl Window {
    pub(crate) fn new(window: Window_w) -> Self {
        Self { window }
    }

    pub fn available_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
        self.window.available_monitors()
    }

    pub fn current_monitor(&self) -> Option<MonitorHandle> {
        self.window.current_monitor()
    }

    pub fn primary_monitor(&self) -> Option<MonitorHandle> {
        self.window.primary_monitor()
    }

    pub fn fullscreen(&self) -> Option<Fullscreen> {
        self.window.fullscreen()
    }

    pub fn set_fullscreen(&mut self, fullscreen: Option<Fullscreen>) {
        self.window.set_fullscreen(fullscreen);
    }

    /// Return the resolution of the window.
    pub fn resolution(&self) -> (u32, u32) {
        let size = self.window.inner_size();

        (size.width, size.height)
    }

    /// Modifies the resolution of the window.
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.window.set_inner_size(PhysicalSize::new(width, height));
    }

    /// Return the the left-top position of the window.
    pub fn position(&self) -> Result<(i32, i32), NotSupportError> {
        let position = self.window.inner_position()?;

        Ok((position.x, position.y))
    }
    
    /// Set whether the window is resizable or not.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
    }

    /// Modifies the window's visibility.
    ///
    /// If `false`, this will hide the window. If `true`, this will show the window.
    pub fn set_visible(&mut self, visible: bool) {
        self.window.set_visible(visible);
    }

    /// Change whether or not the window will always be on top of other windows.
    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        self.window.set_always_on_top(always_on_top);
    }

    /// Modifies the title of the window.
    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    /// Grabs the cursor, preventing it from leaving the window.
    ///
    /// There's no guarantee that the cursor will be hidden. You should hide it by yourself if you want so.
    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(grab)
    }

    /// Change the position of the cursor in window coordinate.
    pub fn set_cursor_position(&mut self, x: u32, y: u32) -> Result<(), ExternalError> {
        self.window.set_cursor_position(PhysicalPosition::new(x, y))
    }

    /// Modifies the cursor's visible.
    ///
    /// If `false`, this will hide the cursor. If `true`, this will show the cursor.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    // TODO: After finish texture module.
    pub fn set_cursor_icon(&mut self) {
        todo!()
    }

    // TODO: After finish texture module.
    pub fn set_window_icon(&mut self) {
        todo!()
    }
}
