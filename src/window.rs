// use super::input::Input;
// use winit::{
//     dpi::{PhysicalPosition, PhysicalSize},
//     event::{DeviceEvent, Event, KeyboardInput, MouseScrollDelta, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     platform::run_return::EventLoopExtRunReturn,
//     window::WindowBuilder,
// };

// pub type WindowCreateError = winit::error::OsError;

// pub(crate) struct Window {
//     window: winit::window::Window,
//     event_loop: winit::event_loop::EventLoop<()>,
// }

// // TODO: Fix and improve the function
// impl Window {
//     fn new() -> Result<Self, WindowCreateError> {
//         let event_loop = EventLoop::new();
//         let window = WindowBuilder::new().with_title("experiment window").build(&event_loop)?;

//         Ok(Self { window, event_loop })
//     }

//     fn run_return(&mut self, input: &mut Input) {
//         input.prepare();

//         self.event_loop.run_return(|event, _, control_flow| {
//             *control_flow = ControlFlow::Wait;

//             match event {
//                 Event::WindowEvent { event, .. } => match event {
//                     WindowEvent::MouseInput { button, state, .. } => {
//                         input.mouse.set_button_state(
//                             button,
//                             if state == winit::event::ElementState::Pressed {
//                                 true
//                             } else {
//                                 false
//                             },
//                         );
//                     }
//                     WindowEvent::CursorMoved { position, .. } => {
//                         input.mouse.cursor_position = (position.x as u32, position.y as u32);
//                     }
//                     WindowEvent::MouseWheel { delta, .. } => {
//                         if let MouseScrollDelta::LineDelta(x, y) = delta {
//                             input.mouse.wheel_delta = (x, y);
//                         }
//                     }
//                     _ => {}
//                 },
//                 Event::DeviceEvent { event, .. } => match event {
//                     DeviceEvent::MouseMotion { delta } => {
//                         input.mouse.motion_delta = (delta.0 as f32, delta.1 as f32);
//                     }
//                     DeviceEvent::Key(KeyboardInput {
//                         virtual_keycode: Some(vkc),
//                         state,
//                         ..
//                     }) => {
//                         input.keyboard.set_keycode_state(
//                             vkc,
//                             if state == winit::event::ElementState::Pressed {
//                                 true
//                             } else {
//                                 false
//                             },
//                         );
//                     }
//                     _ => {}
//                 },
//                 Event::MainEventsCleared => {
//                     *control_flow = ControlFlow::Exit;
//                 }
//                 _ => {}
//             }
//         });
//     }

//     fn is_fullscreen(&self) -> bool {
//         todo!()
//     }

//     fn get_inner_position(&self) -> (i32, i32) {
//         todo!()
//     }

//     fn get_inner_size(&self) -> (u32, u32) {
//         todo!()
//     }

//     fn get_outer_position(&self) -> (i32, i32) {
//         todo!()
//     }

//     fn get_outer_size(&self) -> (u32, u32) {
//         todo!()
//     }

//     fn get_scale_factor(&self) -> f64 {
//         todo!()
//     }

//     fn set_always_on_top(&mut self, always_on_top: bool) {
//         self.window.set_always_on_top(always_on_top);
//     }

//     fn set_cursor_grab(&mut self, grab: bool) {
//         self.window.set_cursor_grab(grab);
//     }

//     fn set_fullscreen(&mut self, fullscreen: bool) {
//         todo!()
//     }

//     // FIXME: dirty impl
//     fn set_inner_size(&mut self, size: (u32, u32)) {
//         self.window.set_inner_size(PhysicalSize::<u32>::from(size));
//     }

//     // FIXME: dirty impl
//     fn set_max_inner_sie(&mut self, max_size: (u32, u32)) {
//         todo!()
//     }

//     fn set_maximized(&mut self, maximized: bool) {
//         self.window.set_maximized(maximized);
//     }

//     // FIXME: dirty impl
//     fn set_min_inner_size(&mut self, min_size: (u32, u32)) {
//         todo!()
//     }

//     fn set_minimized(&mut self, minimized: bool) {
//         self.window.set_minimized(minimized);
//     }

//     // FIXME: dirty impl
//     fn set_outer_position(&mut self, position: (i32, i32)) {
//         self.window.set_outer_position(PhysicalPosition::<i32>::from(position));
//     }

//     fn set_resizable(&mut self, resizable: bool) {
//         self.window.set_resizable(resizable);
//     }

//     // FIXME: dirty impl
//     fn set_title(&mut self, title: String) {
//         self.window.set_title(&title);
//     }

//     fn set_visible(&mut self, visible: bool) {
//         self.window.set_visible(visible);
//     }
// }

// // TODO: Window模块应该如何设计