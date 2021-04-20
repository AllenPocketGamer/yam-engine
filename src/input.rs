use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, WindowEvent,
};

use crate::{misc::coordinates::Transformation, nalgebra::Vector4};

use std::collections::HashMap;

pub type KeyCode = winit::event::VirtualKeyCode;
pub type MouseButton = winit::event::MouseButton;

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

    pub(crate) fn apply(&mut self, evts: &mut Vec<Event<()>>, trf: &Transformation) {
        self.mouse.before_apply();
        self.keyboard.before_apply();

        self.mouse.trf = *trf;

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
                        self.mouse.cursor_position_ss = (position.x as f32, position.y as f32);
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
                                ElementState::Pressed if *bs != ButtonState::Pressed => {
                                    *bs = ButtonState::JustPressed
                                }
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
                        self.mouse.mouse_motion = (delta.0 as f32, delta.1 as f32);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn release_all(&mut self) {
        self.mouse.release_all();
        self.keyboard.release_all();
    }
}

pub struct Mouse {
    mouse_motion: (f32, f32),
    mouse_wheel_motion: (f32, f32),
    mouse_button_state: HashMap<MouseButton, ButtonState>,

    cursor_state: CursorState,
    // cursor position in `screen space`.
    cursor_position_ss: (f32, f32),

    trf: Transformation,
}

impl Mouse {
    fn new() -> Self {
        Self {
            mouse_motion: (0f32, 0f32),
            mouse_wheel_motion: (0f32, 0f32),
            mouse_button_state: HashMap::with_capacity(4),

            cursor_state: CursorState::Left,
            cursor_position_ss: (0f32, 0f32),

            trf: Transformation::default(),
        }
    }

    /// Detect whether the mouse button has been pressed.
    pub fn pressed(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::Pressed,
            None => false,
        }
    }

    /// Detect whether the mouse button has been released.
    pub fn released(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::Released,
            None => true,
        }
    }

    /// Detect whether the mouse button has just been pressed.  
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::JustPressed,
            None => false,
        }
    }

    /// Detect whether the mouse button has just been released.
    pub fn just_released(&self, button: MouseButton) -> bool {
        match self.mouse_button_state.get(&button) {
            Some(state) => *state == ButtonState::JustReleased,
            None => false,
        }
    }

    /// Detect whether the cursor has been left the window.
    pub fn cursor_left(&self) -> bool {
        self.cursor_state == CursorState::Left
    }

    /// Detect whether the cursor has been entered the window.
    pub fn cursor_entered(&self) -> bool {
        self.cursor_state == CursorState::Entered
    }

    /// Detect whether the cursor has just been left the window.
    pub fn cursor_just_left(&self) -> bool {
        self.cursor_state == CursorState::JustLeft
    }

    /// Detect whether the cursor has just been entered the window.
    pub fn cursor_just_entered(&self) -> bool {
        self.cursor_state == CursorState::JustEntered
    }

    /// Return the position of the cursor in `screen space`.
    pub fn cursor_position_in_ss(&self) -> (f32, f32) {
        self.cursor_position_ss
    }

    /// Return the position of the cursor in `view space`.
    pub fn cursor_position_in_vs(&self) -> (f32, f32) {
        let mp_vs = self.trf.mx_s2v()
            * Vector4::new(
                self.cursor_position_ss.0,
                self.cursor_position_ss.1,
                0.0,
                1.0,
            );

        (mp_vs.x, mp_vs.y)
    }

    /// Return the position of the cursor in `world space`.
    pub fn cursor_position_in_ws(&self) -> (f32, f32) {
        let mp_ws = self.trf.mx_s2w()
            * Vector4::new(
                self.cursor_position_ss.0,
                self.cursor_position_ss.1,
                0.0,
                1.0,
            );

        (mp_ws.x, mp_ws.y)
    }

    /// Return the difference in the position of the mouse between two frames(in screen space).
    pub fn mouse_motion_in_ss(&self) -> (f32, f32) {
        self.mouse_motion
    }

    /// Return the difference in the position of the mouse between two frames(in view space).
    pub fn mouse_motion_in_vs(&self) -> (f32, f32) {
        let mm_vs =
            self.trf.mx_s2v() * Vector4::new(self.mouse_motion.0, self.mouse_motion.1, 0.0, 0.0);

        (mm_vs.x, mm_vs.y)
    }

    /// Return the difference in the position of the mouse between two frames(in world space).
    pub fn mouse_motion_in_ws(&self) -> (f32, f32) {
        let mm_ws =
            self.trf.mx_s2w() * Vector4::new(self.mouse_motion.0, self.mouse_motion.1, 0.0, 0.0);

        (mm_ws.x, mm_ws.y)
    }

    /// Return the difference in the wheel position of the mouse between two frames.
    pub fn mouse_wheel_motion(&self) -> (f32, f32) {
        self.mouse_wheel_motion
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

    fn release_all(&mut self) {
        for (_, bs) in self.mouse_button_state.iter_mut() {
            *bs = ButtonState::Released;
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

    /// Detect whether the keyboard button has been pressed.
    pub fn just_pressed(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::JustPressed,
            None => false,
        }
    }

    /// Detect whether the keyboard button has been released.
    pub fn just_released(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::JustReleased,
            None => false,
        }
    }

    /// Detect whether the keyboard button has just been pressed.
    pub fn pressed(&self, keycode: KeyCode) -> bool {
        match self.key_button_state.get(&keycode) {
            Some(state) => *state == ButtonState::Pressed,
            None => false,
        }
    }

    /// Detect whether the keyboard button has just been released.
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

    fn release_all(&mut self) {
        for (_, bs) in self.key_button_state.iter_mut() {
            *bs = ButtonState::Released;
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
