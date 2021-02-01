use std::thread;
use std::time::Duration;

use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        match event {
            Event::NewEvents(sc) => {
                // println!("{:?}", sc);
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    winit::event::WindowEvent::Resized(_) => {}
                    winit::event::WindowEvent::Moved(_) => {}
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    winit::event::WindowEvent::Destroyed => {}
                    winit::event::WindowEvent::DroppedFile(_) => {}
                    winit::event::WindowEvent::HoveredFile(_) => {}
                    winit::event::WindowEvent::HoveredFileCancelled => {}
                    winit::event::WindowEvent::ReceivedCharacter(_) => {}
                    winit::event::WindowEvent::Focused(_) => {}
                    winit::event::WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {}
                    winit::event::WindowEvent::ModifiersChanged(_) => {}
                    winit::event::WindowEvent::CursorMoved { device_id, position, modifiers } => {}
                    winit::event::WindowEvent::CursorEntered { device_id } => {}
                    winit::event::WindowEvent::CursorLeft { device_id } => {}
                    winit::event::WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {}
                    winit::event::WindowEvent::MouseInput { device_id, state, button, modifiers } => {}
                    winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => {}
                    winit::event::WindowEvent::AxisMotion { device_id, axis, value } => {}
                    winit::event::WindowEvent::Touch(_) => {}
                    winit::event::WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {}
                    winit::event::WindowEvent::ThemeChanged(_) => {}
                }
            }
            Event::DeviceEvent { device_id, event } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {
                // TODO: impl logical when application suspended
            }
            Event::Resumed => {
                // TODO: impl logical when application resumed
            }
            Event::MainEventsCleared => {
            }
            Event::RedrawRequested(_) => {}
            Event::RedrawEventsCleared => {}
            Event::LoopDestroyed => {}
        }
    })
}
