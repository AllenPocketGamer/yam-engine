use legion::storage::ComponentWriter;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

pub(super) struct Window {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub(crate) fn new() -> Result<Self, WindowCreateError> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("experiment window").build(&event_loop)?;

        Ok(Window { window, event_loop })
    }

    pub(crate) fn run_returned(&mut self) -> Input {
        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let Event::WindowEvent {event, ..} = &event {
                println!("{:?}", event);
            }

            match event {
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => {},
            }
        });

        Input::default()
    }
}

#[derive(Clone, Copy)]
pub struct Input {}

impl Input {
    pub fn welcum() -> bool {
        true
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {}
    }
}

pub type WindowCreateError = winit::error::OsError;
