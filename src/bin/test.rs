use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

fn main() {
    let mut event_loop = EventLoop::new();
    let _window = WindowBuilder::new().with_title("experiment window").build(&event_loop).unwrap();

    loop {
        let mut count = 0;

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            count += 1;

            match event {
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => {},
            }
        });
        
        println!("count: {}", count);
        
        count = 0;
    }
}
