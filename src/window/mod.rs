use std::collections::HashMap;

use winit::{
    event::{DeviceEvent, Event, KeyboardInput, MouseScrollDelta, WindowEvent},
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

        Ok(Self { window, event_loop })
    }

    pub(crate) fn run_return(&mut self, input: &mut Input) {
        input.prepare();

        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::MouseInput { button, state, .. } => {
                        input.mouse.set_button_state(
                            button,
                            if state == winit::event::ElementState::Pressed {
                                true
                            } else {
                                false
                            },
                        );
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        input.mouse.position = (position.x as u32, position.y as u32);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let MouseScrollDelta::LineDelta(x, y) = delta {
                            input.mouse.wheel_delta = (x, y);
                        }
                    }
                    _ => {}
                },
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        input.mouse.motion_delta = (delta.0 as f32, delta.1 as f32);
                    }
                    DeviceEvent::Key(KeyboardInput {
                        virtual_keycode: Some(vkc),
                        state,
                        ..
                    }) => {
                        input.keyboard.set_keycode_state(
                            vkc,
                            if state == winit::event::ElementState::Pressed {
                                true
                            } else {
                                false
                            },
                        );
                    }
                    _ => {}
                },
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }
}

pub struct Input {
    pub mouse: Mouse,
    pub keyboard: Keyboard,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.mouse.prepare();
        self.keyboard.prepare();
    }
}

pub struct Mouse {
    pub(crate) position: (u32, u32),
    pub(crate) window_position: (u32, u32),
    pub(crate) motion_delta: (f32, f32),
    pub(crate) wheel_delta: (f32, f32),
    button_to_state: HashMap<MouseButton, ElementState>,
}

impl Mouse {
    fn new() -> Self {
        Self {
            position: (0, 0),
            motion_delta: (0f32, 0f32),
            wheel_delta: (0f32, 0f32),
            window_position: (0, 0),
            button_to_state: HashMap::with_capacity(3),
        }
    }

    fn prepare(&mut self) {
        self.motion_delta = Default::default();
        self.wheel_delta = Default::default();
        self.button_to_state.iter_mut().map(|(_k, v)| v).for_each(|v| match *v {
            ElementState::JustPressed => *v = ElementState::Pressed,
            ElementState::JustReleased => *v = ElementState::Released,
            _ => {}
        });
    }

    fn set_button_state(&mut self, button: MouseButton, is_pressed: bool) {
        let target_state = if is_pressed {
            ElementState::JustPressed
        } else {
            ElementState::JustReleased
        };

        match self.button_to_state.get_mut(&button) {
            Some(state) => {
                *state = target_state;
            }
            None => {
                self.button_to_state.insert(button, target_state);
            }
        }
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        match self.button_to_state.get(&button) {
            Some(state) => *state == ElementState::JustPressed,
            None => false,
        }
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        match self.button_to_state.get(&button) {
            Some(state) => *state == ElementState::JustReleased,
            None => false,
        }
    }

    pub fn pressed(&self, button: MouseButton) -> bool {
        match self.button_to_state.get(&button) {
            Some(state) => *state == ElementState::Pressed,
            None => false,
        }
    }

    pub fn released(&self, button: MouseButton) -> bool {
        match self.button_to_state.get(&button) {
            Some(state) => *state == ElementState::Released,
            None => true,
        }
    }

    pub fn get_position_at_window(&self) -> (u32, u32) {
        self.position
    }

    pub fn get_position_at_desktop(&self) -> (u32, u32) {
        (self.position.0 + self.window_position.0, self.position.1 + self.window_position.1)
    }

    pub fn get_motion_delta(&self) -> (f32, f32) {
        self.motion_delta
    }

    pub fn get_wheel_delta(&self) -> (f32, f32) {
        self.wheel_delta
    }
}

pub struct Keyboard {
    keycode_to_state: HashMap<KeyCode, ElementState>,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            keycode_to_state: HashMap::with_capacity(16),
        }
    }

    fn prepare(&mut self) {
        self.keycode_to_state.iter_mut().map(|(_k, e)| e).for_each(|e| match *e {
            ElementState::JustPressed => *e = ElementState::Pressed,
            ElementState::JustReleased => *e = ElementState::Released,
            _ => {}
        });
    }

    fn set_keycode_state(&mut self, keycode: KeyCode, is_pressed: bool) {
        match self.keycode_to_state.get_mut(&keycode) {
            Some(state) => {
                match *state {
                    ElementState::Pressed if !is_pressed => *state = ElementState::JustReleased, 
                    ElementState::Released if is_pressed => *state = ElementState::JustPressed,
                    _ => {},
                }
            }
            None => {
                self.keycode_to_state.insert(
                    keycode,
                    if is_pressed {
                        ElementState::JustPressed
                    } else {
                        ElementState::JustReleased
                    },
                );
            }
        }
    }

    pub fn just_pressed(&self, keycode: KeyCode) -> bool {
        match self.keycode_to_state.get(&keycode) {
            Some(state) => *state == ElementState::JustPressed,
            None => false,
        }
    }

    pub fn just_released(&self, keycode: KeyCode) -> bool {
        match self.keycode_to_state.get(&keycode) {
            Some(state) => *state == ElementState::JustReleased,
            None => false,
        }
    }

    pub fn pressed(&self, keycode: KeyCode) -> bool {
        match self.keycode_to_state.get(&keycode) {
            Some(state) => *state == ElementState::Pressed,
            None => false,
        }
    }

    pub fn released(&self, keycode: KeyCode) -> bool {
        match self.keycode_to_state.get(&keycode) {
            Some(state) => *state == ElementState::Released,
            None => true,
        }
    }
}

pub type WindowCreateError = winit::error::OsError;

pub type MouseButton = winit::event::MouseButton;
pub type KeyCode = winit::event::VirtualKeyCode;

#[derive(Clone, Copy, PartialEq)]
enum ElementState {
    JustPressed,
    JustReleased,
    Pressed,
    Released,
}
