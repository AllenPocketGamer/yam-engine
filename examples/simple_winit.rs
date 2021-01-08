use event_loop::EventLoop;
use window::Window;
use winit::*;

fn main() {
    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop);
}