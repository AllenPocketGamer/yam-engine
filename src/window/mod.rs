
pub(crate) struct EventLoop {
    event_loop: winit::event_loop::EventLoop<()>,
}

impl EventLoop {
    pub(crate) fn new() -> Self {
        EventLoop {
            event_loop: winit::event_loop::EventLoop::new(),
        }
    }

    pub(crate) fn refresh(&mut self) -> Input {
        println!("refresh");
        Input {}
    }
}

pub struct Window {
    window: winit::window::Window,
    pub is_quit: bool,
}

impl Window {
    pub(crate) fn new(event_loop: &EventLoop) -> Result<Window, WindowCreateError> {
        let window = winit::window::Window::new(&event_loop.event_loop);

        match window {
            Ok(window) => Ok(Window {window, is_quit: false}),
            Err(e) => Err(e)
        }
    }

    pub fn set_position(&self) {
        println!("call set position");
    }
    
    pub fn quit(&mut self) {
        self.is_quit = true;
    }
}

pub struct WindowSettings {

}

pub struct Input {

}

impl Input {
    
}

pub type WindowCreateError = winit::error::OsError;