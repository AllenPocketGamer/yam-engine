use std::collections::HashMap;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, WindowEvent};

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

    pub(crate) fn apply(&mut self, evts: &mut Vec<Event<()>>) {
        self.mouse.before_apply();
        self.keyboard.before_apply();

        for evt in evts.drain(..) {
            match evt {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::MouseInput { button, state, .. } => {
                        if let Some(bs) = self.mouse.mouse_button_state.get_mut(&button) {
                            match state {
                                ElementState::Pressed => *bs = ButtonState::JustPressed,
                                ElementState::Released => *bs = ButtonState::JustReleased,
                            };
                        } else {
                            self.mouse.mouse_button_state.insert(
                                button,
                                match state {
                                    ElementState::Pressed => ButtonState::JustPressed,
                                    ElementState::Released => ButtonState::JustReleased,
                                },
                            );
                        }
                    }
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(x, y),
                        ..
                    } => {
                        self.mouse.mouse_wheel_motion = (x, y);
                    }

                    WindowEvent::CursorLeft { .. } => {
                        self.mouse.cursor_state = CursorState::JustLeft;
                    }
                    WindowEvent::CursorEntered { .. } => {
                        self.mouse.cursor_state = CursorState::JustEntered;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.mouse.cursor_position = (position.x as f32, position.y as f32);
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => {
                        if let Some(bs) = self.keyboard.key_button_state.get_mut(&keycode) {
                            match state {
                                ElementState::Pressed if *bs != ButtonState::Pressed => *bs = ButtonState::JustPressed,
                                ElementState::Released => *bs = ButtonState::JustReleased,
                                _ => {}
                            }
                        } else {
                            self.keyboard.key_button_state.insert(
                                keycode,
                                match state {
                                    ElementState::Pressed => ButtonState::JustPressed,
                                    ElementState::Released => ButtonState::JustReleased,
                                },
                            );
                        }
                    }

                    _ => {}
                },
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        self.mouse.mouse_motion = (delta.0 as f32, delta.0 as f32);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

// FIXME: change tuple to vec2 after finishing math module
pub struct Mouse {
    mouse_motion: (f32, f32),
    mouse_wheel_motion: (f32, f32),
    mouse_button_state: HashMap<MouseButton, ButtonState>,

    cursor_state: CursorState,
    cursor_position: (f32, f32),
}

impl Mouse {
    fn new() -> Self {
        Self {
            mouse_motion: (0f32, 0f32),
            mouse_wheel_motion: (0f32, 0f32),
            mouse_button_state: HashMap::with_capacity(4),

            cursor_state: CursorState::Left,
            cursor_position: (0f32, 0f32),
        }
    }

    pub fn pressed(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::Pressed,
            None => false,
        }
    }

    pub fn released(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::Released,
            None => true,
        }
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::JustPressed,
            None => false,
        }
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::JustReleased,
            None => false,
        }
    }

    pub fn cursor_left(&self) -> bool {
        self.cursor_state == CursorState::Left
    }

    pub fn cursor_entered(&self) -> bool {
        self.cursor_state == CursorState::Entered
    }

    pub fn cursor_just_left(&self) -> bool {
        self.cursor_state == CursorState::JustLeft
    }

    pub fn cursor_just_entered(&self) -> bool {
        self.cursor_state == CursorState::JustEntered
    }

    // FIXME: change(u32, u32) to vec2 after finishing math module
    pub fn cursor_position(&self) -> (f32, f32) {
        self.cursor_position
    }

    fn before_apply(&mut self) {
        self.mouse_motion = (0f32, 0f32);
        self.mouse_wheel_motion = (0f32, 0f32);

        self.cursor_state = match self.cursor_state {
            CursorState::JustLeft => CursorState::Left,
            CursorState::JustEntered => CursorState::Entered,
            _ => self.cursor_state,
        };

        for bs in self.mouse_button_state.values_mut() {
            match *bs {
                ButtonState::JustPressed => *bs = ButtonState::Pressed,
                ButtonState::JustReleased => *bs = ButtonState::Released,
                _ => {}
            };
        }
    }
}

pub struct Keyboard {
    key_button_state: HashMap<KeyCode, ButtonState>,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            key_button_state: HashMap::with_capacity(16),
        }
    }

    pub fn just_pressed(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::JustPressed,
            None => false,
        }
    }

    pub fn just_released(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::JustReleased,
            None => false,
        }
    }

    pub fn pressed(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::Pressed,
            None => false,
        }
    }

    pub fn released(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::Released,
            None => true,
        }
    }

    fn before_apply(&mut self) {
        for bs in self.key_button_state.values_mut() {
            match *bs {
                ButtonState::JustPressed => *bs = ButtonState::Pressed,
                ButtonState::JustReleased => *bs = ButtonState::Released,
                _ => {}
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ButtonState {
    Pressed,
    Released,
    JustPressed,
    JustReleased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorState {
    Left,
    Entered,
    JustLeft,
    JustEntered,
}

pub type MouseButton = winit::event::MouseButton;
pub type KeyCode = winit::event::VirtualKeyCode;
